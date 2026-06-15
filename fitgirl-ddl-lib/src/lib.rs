use std::sync::{LazyLock, OnceLock};

use http::HeaderValue;
use wreq::Client;

pub mod errors;
pub mod extract;
pub mod scrape;

pub use http;
pub use wreq::{Request, RequestBuilder};

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    wreq::Client::builder()
        .emulation(wreq_util::Emulation::Firefox148)
        .build()
        .unwrap()
});
pub static FITGIRL_COOKIES: OnceLock<HeaderValue> = OnceLock::new();

/// Accepts cookies in form like `name1=value1; name=value2; ...`,
/// Also see [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Cookie).
///
/// Once cookies for fitgirl was initialized, this function returns Err(cookies)
pub fn set_fg_cookies(cookies: impl Into<HeaderValue>) -> Result<(), HeaderValue> {
    FITGIRL_COOKIES.set(cookies.into())
}
