use std::sync::OnceLock;

use nyquest::{AsyncClient, client::BuildClientResult};

pub mod errors;
pub mod extract;
pub mod scrape;

pub static NYQUEST_CLIENT: OnceLock<AsyncClient> = OnceLock::new();

pub async fn init_nyquest() -> BuildClientResult<()> {
    let async_client = nyquest::ClientBuilder::default()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0",
        )
        .build_async()
        .await?;
    _ = NYQUEST_CLIENT.set(async_client);
    Ok(())
}
