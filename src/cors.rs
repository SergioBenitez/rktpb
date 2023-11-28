use std::io::Cursor;
use std::collections::HashMap;

use rocket::{Rocket, Request, Response, Orbit};
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::http::{Status, Header, Method, uri::Absolute};
use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Cors {
    #[serde(default)]
    cors: HashMap<Absolute<'static>, Vec<Method>>,
}

impl Cors {
    pub fn fairing() -> impl Fairing {
        AdHoc::try_on_ignite("CORS Configuration", |rocket| async {
            match rocket.figment().extract::<Cors>() {
                Ok(cors) => Ok(rocket.attach(cors)),
                Err(e) => {
                    let kind = rocket::error::ErrorKind::Config(e);
                    rocket::Error::from(kind).pretty_print();
                    Err(rocket)
                },
            }
        })
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info { name: "CORS", kind: Kind::Liftoff | Kind::Response }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        use yansi::Paint;

        info!("{}{}", "📫 ".mask(), "CORS:".magenta());
        if self.cors.is_empty() {
            return info_!("status: {}", "disabled".red());
        }

        info_!("status: {}", "enabled".green());
        for (host, methods) in &self.cors {
            info_!("{}: {:?}", host.magenta(), methods.primary());
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let allowed_host_methods = req.headers().get_one("Origin")
            .and_then(|origin| Absolute::parse(origin).ok())
            .and_then(|host| self.cors.get_key_value(&host))
            .filter(|(_, methods)| methods.contains(&req.method()));

        if let Some((host, methods)) = allowed_host_methods {
            const ALLOW_ORIGIN: &str = "Access-Control-Allow-Origin";
            const ALLOW_METHODS: &str = "Access-Control-Allow-Methods";
            const ALLOW_HEADERS: &str = "Access-Control-Allow-Headers";

            let mut allow_methods = String::with_capacity(methods.len() * 8);
            for (i, method) in methods.iter().enumerate() {
                if i != 0 { allow_methods.push(','); }
                allow_methods.push_str(method.as_str());
            }

            res.set_header(Header::new(ALLOW_ORIGIN, host.to_string()));
            res.set_header(Header::new(ALLOW_METHODS, allow_methods));
            res.set_header(Header::new(ALLOW_HEADERS, "Content-Type"));

            if req.method() == Method::Options && res.status() == Status::NotFound {
                res.set_status(Status::Ok);
                res.set_sized_body(0, Cursor::new(""));
            }
        }
    }
}
