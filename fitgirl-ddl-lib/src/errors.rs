use scraper::error::SelectorErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrapeError {
    #[error("fuckingfast.co source was missing")]
    FuckingFastSourceMissing,
    #[error("ill-formed url")]
    IllFormedURL(#[from] url::ParseError),
    #[error("expected link to single game description")]
    UnexpectedURL,
    #[error("request: {0}")]
    RequestError(#[from] nyquest::Error),
    #[error("join error")]
    JoinError,
    #[error("invalid css selector")]
    InvalidCSSSelector,
}

impl From<SelectorErrorKind<'_>> for ScrapeError {
    fn from(_: SelectorErrorKind<'_>) -> Self {
        Self::InvalidCSSSelector
    }
}

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("filename was not found")]
    FilenameMissing,
    #[error("direct download link was not found")]
    DDLMissing,
    #[error("request: {0}")]
    RequestError(#[from] nyquest::Error),
    #[error("invalid css selector")]
    InvalidCSSSelector,
    #[error("join error")]
    JoinError,
    #[error("rate limited")]
    RateLimited,
    #[error("file was deleted")]
    FileNotFound,
}

impl From<SelectorErrorKind<'_>> for ExtractError {
    fn from(_: SelectorErrorKind<'_>) -> Self {
        Self::InvalidCSSSelector
    }
}
