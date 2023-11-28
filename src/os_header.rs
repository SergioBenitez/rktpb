use rocket::request::{Request, FromRequest, Outcome};
use rocket::http::Status;

pub struct ClientOs(&'static str);

impl ClientOs {
    pub fn name(&self) -> &'static str {
        self.0
    }
}

static AGENT_MAP: &[(&str, &str)] = &[
    ("Windows", "windows"), ("OpenBSD", "bsd"), ("SunOS", "sun"),
    ("Macintosh", "darwin"), ("Linux", "linux"), ("Mac OS X", "darwin"),
    ("Google", "google"), ("Yahoo", "yahoo"), ("Bing", "bing")
];

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientOs {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<ClientOs, ()> {
        if let Some(agent) = req.headers().get_one("User-Agent") {
            for &(agent_os, os) in AGENT_MAP {
                if agent.contains(agent_os) {
                    return Outcome::Success(ClientOs(os));
                }
            }

            return Outcome::Forward(Status::NotFound);
        }

        Outcome::Forward(Status::NotFound)
    }
}
