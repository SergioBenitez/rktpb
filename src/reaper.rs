use std::io;
use std::time::Duration;
use std::path::PathBuf;

use rocket::tokio;

use rocket::{Rocket, Orbit};
use rocket::serde::Deserialize;
use rocket::fairing::{Kind, Fairing, AdHoc, Info};

/// Default max age is 30 days.
const DEFAULT_MAX_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Default repeat interval is 5 minutes.
const DEFAULT_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Minimum allowed interval is 30 seconds.
const MIN_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Reaper {
    #[serde(default, with = "humantime_serde")]
    max_age: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    reap_interval: Option<Duration>,
    upload_dir: PathBuf,
}

impl Reaper {
    pub fn fairing() -> impl Fairing {
        AdHoc::try_on_ignite("Reaper Config", |rocket| async {
            match rocket.figment().extract::<Self>() {
                Ok(reaper) => Ok(rocket.attach(reaper)),
                Err(e) => {
                    let kind = rocket::error::ErrorKind::Config(e);
                    rocket::Error::from(kind).pretty_print();
                    Err(rocket)
                },
            }
        })
    }

    async fn reap(&self) -> io::Result<(usize, usize, usize)> {
        let mut files_reaped = 0;
        let mut files_checked = 0;
        let mut files_ok = 0;
        let mut entries = tokio::fs::read_dir(&self.upload_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(file_name) => file_name.to_string_lossy(),
                None => continue,
            };

            let metadata = entry.metadata().await?;
            if !metadata.is_file() || path.extension().is_some() || file_name.starts_with('.') {
                continue;
            }

            let last_access = metadata.accessed()?;
            let elapsed = last_access.elapsed().unwrap_or(Duration::MAX);
            if elapsed >= self.max_age() {
                files_reaped += tokio::fs::remove_file(&path).await.is_ok() as usize;
            } else {
                files_ok += 1;
            }

            files_checked += 1;
        }

        Ok((files_reaped, files_checked, files_ok))
    }

    fn max_age(&self) -> Duration {
        self.max_age.unwrap_or(DEFAULT_MAX_AGE)
    }

    fn interval(&self) -> Duration {
        let base = self.reap_interval.unwrap_or(DEFAULT_INTERVAL);
        std::cmp::max(MIN_INTERVAL, base)
    }
}

#[rocket::async_trait]
impl Fairing for Reaper {
    fn info(&self) -> rocket::fairing::Info {
        Info { name: "Reaper", kind: Kind::Liftoff }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        use yansi::Paint;

        let icon = "💀 ".mask();
        info!("{}{}", icon, "Reaper:".magenta());
        info_!("status: {}", "enabled".green());
        info_!("directory: {}", self.upload_dir.display().magenta());
        info_!("max age: {}", humantime::format_duration(self.max_age()).magenta());
        info_!("interval: {}", humantime::format_duration(self.interval()).magenta());

        if let Some(i) = self.reap_interval {
            if i < MIN_INTERVAL {
                let f = humantime::format_duration(i);
                warn_!("note: interval minimum is 30s (was {})", f.magenta());
            }
        }

        let reaper = self.clone();
        tokio::spawn(async move {
            loop {
                match reaper.reap().await {
                    Err(e) => warn!("Error encountered while reaping: {}", e),
                    Ok((r, n, k)) => {
                        info!("{}reaper: {}/{} files reaped ({} fresh)", icon, r, n, k)
                    }
                }

                tokio::time::sleep(reaper.interval()).await;
            }
        });
    }
}
