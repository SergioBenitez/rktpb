use std::fmt;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use rand::distributions::DistString;
use rand::{thread_rng, distributions::Alphanumeric};

use rocket::http::{RawStr, ContentType, impl_from_uri_param_identity};
use rocket::http::uri::{self, fmt::UriDisplay};
use rocket::request::FromParam;
use rocket::serde::{Serialize, Serializer};

use crate::Config;

/// The (id, extension) of the requested paste.
pub struct PasteId<'a>(Cow<'a, str>, Option<&'a str>);

impl<'a> PasteId<'a> {
    /// Generates a new, random paste ID.
    pub fn new(config: &Config) -> PasteId<'static> {
        PasteId::with_ext(config, None)
    }

    /// Randomly generates an ID of the configured length. There are no
    /// requirements on `ext`; it used simply as a hint in the responder.
    pub fn with_ext<E: Into<Option<&'a str>>>(config: &Config, ext: E) -> Self {
        let id = Alphanumeric.sample_string(&mut thread_rng(), config.id_length);
        PasteId(Cow::Owned(id), ext.into())
    }

    /// The extension of the paste ID, if there is any.
    pub fn ext(&self) -> Option<&str> {
        self.1
    }

    /// The Content-Type of the paste ID based on the extension, if any.
    pub fn content_type(&self) -> Option<ContentType> {
        fn is_browser_executable(ct: &ContentType) -> bool {
            ct.is_html() || ct.is_javascript() || ct.is_css()
        }

        match self.ext().and_then(ContentType::from_extension) {
            Some(ref ct) if is_browser_executable(ct) => None,
            other => other
        }
    }

    /// Where the paste with this ID should be stored.
    pub fn file_path(&self, config: &Config) -> PathBuf {
        config.upload_dir.relative().join(Path::new(&*self.0))
    }
}

impl<'a> FromParam<'a> for PasteId<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        fn valid_id(id: &str) -> bool {
            id.chars().all(char::is_alphanumeric)
        }

        let (id, ext) = RawStr::new(param).split_at_byte(b'.');
        if !valid_id(id.as_str()) {
            return Err(param);
        }

        let (id, ext) = (id.as_str(), (!ext.is_empty()).then(|| ext.as_str()));
        Ok(PasteId(id.into(), ext))
    }
}

impl Serialize for PasteId<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.0)
    }
}

impl UriDisplay<uri::fmt::Path> for PasteId<'_> {
    fn fmt(&self, f: &mut uri::fmt::Formatter<'_, uri::fmt::Path>) -> fmt::Result {
        self.0.fmt(f)?;
        if let Some(ext) = self.1 {
            f.write_raw(".")?;
            ext.fmt(f)?;
        }

        Ok(())
    }
}

impl_from_uri_param_identity!([uri::fmt::Path] ('a) PasteId<'a>);
