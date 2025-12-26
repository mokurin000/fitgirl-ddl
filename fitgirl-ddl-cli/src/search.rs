use std::error::Error;

use compio::runtime::spawn_blocking;
use fitgirl_ddl_lib::NYQUEST_CLIENT;
use nyquest_preset::nyquest::r#async::Request;
use scraper::{Html, Selector};

#[derive(Debug, PartialEq, Eq)]
pub struct SearchEntry {
    pub title: String,
    pub href: String,
}

pub async fn search_games(
    query: &str,
    page: usize,
) -> Result<Vec<SearchEntry>, Box<dyn Error + Send + Sync>> {
    let client = NYQUEST_CLIENT.get().expect("need client initialization");

    let url = format!("https://fitgirl-repacks.site/page/{page}/?s={query}");
    let resp = client.request(Request::get(url)).await?.text().await?;

    Ok(spawn_blocking(move || {
        let html = Html::parse_document(&resp);
        let selector = Selector::parse("h1.entry-title > a").unwrap();

        let atags = html.select(&selector);
        atags
            .map(|tag| {
                let title = tag.text().collect();
                let href = tag.attr("href").unwrap().to_string();
                SearchEntry { title, href }
            })
            .filter(|SearchEntry { title, .. }| !title.starts_with("Updates Digest"))
            .collect()
    })
    .await
    .unwrap())
}
