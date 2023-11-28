#[macro_use]
extern crate rocket;

mod paste_id;
mod os_header;
mod highlight;
mod cors;
mod config;
mod reaper;

use rocket_dyn_templates::{Template, context};
use rocket::tokio::fs::{self, File};
use rocket::tokio::io::{self, AsyncReadExt};

use rocket::State;
use rocket::form::Form;
use rocket::data::Capped;
use rocket::fairing::AdHoc;
use rocket::response::Redirect;
use rocket::request::FlashMessage;
use rocket::fs::{FileServer, TempFile};
use rocket::http::{Status, ContentType};

use paste_id::PasteId;
use os_header::ClientOs;
use highlight::{Highlighter, HIGHLIGHT_EXTS};
use config::Config;
use cors::Cors;
use reaper::Reaper;

#[derive(Responder)]
pub enum Paste {
    Highlighted(Template),
    Regular(File, ContentType),
    Markdown(Template),
}

#[derive(Debug, FromForm)]
struct PasteForm<'r> {
    #[field(validate = len(1..))]
    content: Capped<TempFile<'r>>,
    #[field(validate = with(|e| Highlighter::contains(e), "unknown extension"))]
    ext: &'r str,
}

#[post("/", data = "<paste>")]
async fn upload(
    mut paste: Capped<TempFile<'_>>,
    config: &Config,
) -> io::Result<(Status, String)> {
    let id = PasteId::new(config);
    paste.persist_to(id.file_path(config)).await?;

    let paste_uri = uri!(config.server_url.clone(), get(id));
    let status = match paste.is_complete() {
        true => Status::Created,
        false => Status::PartialContent,
    };

    Ok((status, paste_uri.to_string()))
}

#[post("/web", data = "<form>")]
async fn web_form_submit(
    mut form: Form<PasteForm<'_>>,
    config: &Config,
) -> io::Result<Redirect> {
    let id = PasteId::with_ext(config, form.ext);
    form.content.persist_to(&id.file_path(config)).await?;
    Ok(Redirect::to(uri!(get(id))))
}

// TODO: Authenticate a delete using some kind of token.
#[delete("/<id>")]
async fn delete(id: PasteId<'_>, config: &Config) -> Option<&'static str> {
    fs::remove_file(&id.file_path(config)).await.map(|_| "deleted\n").ok()
}

#[get("/<id>")]
async fn get(
    id: PasteId<'_>,
    highlighter: &State<Highlighter>,
    config: &Config,
) -> io::Result<Option<Paste>> {
    let Ok(mut file) = File::open(&id.file_path(config)).await else {
        return Ok(None)
    };

    let paste = match id.ext() {
        Some("md" | "mdown" | "markdown") => {
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents).await?;
            let content = highlighter.render_markdown(&file_contents)?;
            Paste::Markdown(Template::render("markdown", context! { config, id, content }))
        }
        Some(ext) if Highlighter::contains(ext) => {
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents).await?;
            let content = highlighter.highlight(&file_contents, ext)?;
            let lines = content.lines().count();
            Paste::Highlighted(Template::render("code", context! { config, id, content, lines }))
        }
        _ => Paste::Regular(file, id.content_type().unwrap_or(ContentType::Plain)),
    };

    Ok(Some(paste))
}

#[get("/")]
fn index(config: &Config, os: Option<ClientOs>) -> Template {
    let (os, cmd) = match os.map(|os| os.name()) {
        Some(os @ "windows") => (os, "PowerShell"),
        Some(os @ ("linux" | "darwin")) => (os, "cURL"),
        _ => ("unix", "cURL")
    };

    Template::render("index", context! { config, cmd, os })
}

#[get("/web")]
fn web_form(config: &Config, flash: Option<FlashMessage>) -> Template {
    Template::render("new", context! {
        config,
        extensions: HIGHLIGHT_EXTS,
        error: flash.as_ref().map(FlashMessage::message),
    })
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::custom(Config::figment())
        .mount("/", routes![index, upload, get, delete, web_form, web_form_submit])
        .mount("/", FileServer::from("static").rank(-20))
        .manage(Highlighter::default().expect("failed to load syntax highlighter"))
        .attach(Template::fairing())
        .attach(AdHoc::config::<Config>())
        .attach(Reaper::fairing())
        .attach(Cors::fairing())
}
