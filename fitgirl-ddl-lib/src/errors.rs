use scraper::error::SelectorErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScrapeError {
    #[error("IP 被 ddos-guard 风控")]
    DDoSGuarded,
    #[error("无 fuckingfast 下载源")]
    FuckingFastSourceMissing,
    #[error("URL 格式错误: {0}")]
    IllFormedURI(#[from] http::uri::InvalidUri),
    #[error("请提供单个游戏的 URL")]
    UnexpectedURL,
    #[error("请求错误: {0}")]
    RequestError(String),
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
    RequestError(String),
    #[error("invalid css selector")]
    InvalidCSSSelector,
    #[error("ill-formed uri: {0}")]
    IllFormedURI(#[from] http::uri::InvalidUri),
    #[error("join error")]
    JoinError,
    #[error("rate limited")]
    RateLimited,
    #[error("file was deleted")]
    FileNotFound(String),
}

impl From<SelectorErrorKind<'_>> for ExtractError {
    fn from(_: SelectorErrorKind<'_>) -> Self {
        Self::InvalidCSSSelector
    }
}
