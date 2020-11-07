pub use crate::{
    api::HltvApi,
    https_client::{impls::attohttpc_impl::AttoHttpcImpl, HttpsClient},
};

mod api;
mod https_client;

/// Default HLTV URL.
pub const HLTV_URL: &'static str = "https://www.hltv.org";

pub type Result<T> = std::result::Result<T, Error>;

/// General error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTPS lib error: {0}")]
    HttpsClient(#[source] Box<dyn std::error::Error>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSS parse error: {0}")]
    CssParse(String),

    #[error("HLTV parse error: {0}")]
    HltvParse(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl<'i, E: 'i + std::fmt::Debug> From<cssparser::ParseError<'i, E>> for Error {
    fn from(v: cssparser::ParseError<'i, E>) -> Self {
        Self::CssParse(format!("(cssparser::ParseError) {:?}", v))
    }
}

/// Convert `Option::None` to this crate `Error::String`.
trait NoneErrorExt<T> {
    fn css_err<E: Into<String>>(self, c: E) -> Result<T>;
    fn hltv_parse_err<E: Into<String>>(self, c: E) -> Result<T>;
}

impl<T> NoneErrorExt<T> for Option<T> {
    fn css_err<E: Into<String>>(self, text: E) -> Result<T> {
        self.ok_or_else(|| Error::CssParse(text.into()))
    }

    fn hltv_parse_err<E: Into<String>>(self, text: E) -> Result<T> {
        self.ok_or_else(|| Error::HltvParse(text.into()))
    }
}
