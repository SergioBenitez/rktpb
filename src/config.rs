use rocket::figment::{Figment, Profile};
use rocket::figment::providers::{Format, Toml, Serialized, Env};
use rocket::figment::value::magic::RelativePathBuf;
use rocket::serde::{de, Deserialize, Serialize};

use rocket::{Sentinel, Rocket, Ignite, Request};
use rocket::data::{ByteUnit, ToByteUnit, Limits};
use rocket::http::{Status, uri::Absolute};
use rocket::request::{FromRequest, Outcome};
use rocket::outcome::IntoOutcome;

use yansi::Paint;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub id_length: usize,
    pub paste_limit: ByteUnit,
    pub server_url: Absolute<'static>,
    #[serde(deserialize_with = "directory")]
    #[serde(serialize_with = "RelativePathBuf::serialize_original")]
    pub upload_dir: RelativePathBuf,
    pub source_code_url: String,
}

impl Config {
    pub fn figment() -> Figment {
        #[cfg(debug_assertions)] const DEFAULT_PROFILE: &str = "debug";
        #[cfg(not(debug_assertions))] const DEFAULT_PROFILE: &str = "release";

        // This the base figment, without our `Config` defaults.
        let mut figment = Figment::new()
            .join(rocket::Config::default())
            .merge(Toml::file(Env::var_or("PASTE_CONFIG", "Paste.toml")).nested())
            .merge(Env::prefixed("PASTE_").profile(Profile::Global))
            .select(Profile::from_env_or("PASTE_PROFILE", DEFAULT_PROFILE));

        // Dynamically determine `server_url` default based on address/port.
        let default_server_url = match figment.extract::<rocket::Config>() {
            Ok(config) => {
                let proto = if config.tls_enabled() { "https" } else { "http" };
                let url = format!("{}://{}:{}", proto, config.address, config.port);
                Absolute::parse_owned(url).expect("default URL is Absolute")
            },
            Err(_) => uri!("http://127.0.0.1:8017"),
        };

        // Now set the `Config` defaults.
        figment = figment
            .join(Serialized::defaults(Config {
                id_length: 3,
                paste_limit: 384.kibibytes(),
                server_url: default_server_url,
                upload_dir: "upload".into(),
                source_code_url: String::from("https://github.com/SergioBenitez/rktpb"),
            }));

        // Configure Rocket based on `Config` settings. If this fails now, it's
        // fine - it'll fail when attached too, so we won't miss out.
        if let Ok(config) = figment.extract::<Self>() {
            figment = figment
                .merge((rocket::Config::TEMP_DIR, config.upload_dir))
                .merge((rocket::Config::LIMITS, Limits::default()
                    .limit("form", config.paste_limit)
                    .limit("data-form", config.paste_limit)
                    .limit("file", config.paste_limit)
                    .limit("string", config.paste_limit)
                    .limit("bytes", config.paste_limit)
                    .limit("json", config.paste_limit)
                    .limit("msgpack", config.paste_limit)
                    .limit("paste", config.paste_limit),
                ));
        }

        figment
    }
}

impl Sentinel for Config {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        rocket.state::<Self>().is_none()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Config {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        req.rocket().state::<Config>().or_error((Status::InternalServerError, ()))
    }
}

fn directory<'de, D: de::Deserializer<'de>>(de: D) -> Result<RelativePathBuf, D::Error> {
    let path = RelativePathBuf::deserialize(de)?;
    let resolved = path.relative();
    if !resolved.exists() {
        let path = resolved.display();
        return Err(de::Error::custom(format!("Path {} does not exist.", path.primary())));
    }

    if !resolved.is_dir() {
        let path = resolved.display();
        return Err(de::Error::custom(format!("Path {} is not a directory.", path.primary())));
    }

    Ok(path)
}
