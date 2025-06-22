use std::sync::OnceLock;

use nyquest::AsyncClient;

pub mod errors;
pub mod extract;
pub mod scrape;

pub static NYQUEST_CLIENT: OnceLock<AsyncClient> = OnceLock::new();
pub static FITGIRL_COOKIES: OnceLock<String> = OnceLock::new();

/// Initializes nyquest client.
pub async fn init_nyquest() -> nyquest::Result<()> {
    let async_client = nyquest::ClientBuilder::default()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0",
        )
        .build_async()
        .await?;
    _ = NYQUEST_CLIENT.set(async_client);
    Ok(())
}

/// Accepts cookies in form like `name1=value1; name=value2; ...`,
/// Also see [MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Cookie).
///
/// Once cookies for fitgirl was initialized, this function returns Err(cookies)
pub fn set_fg_cookies(cookies: impl Into<String>) -> Result<(), String> {
    FITGIRL_COOKIES.set(cookies.into())
}
