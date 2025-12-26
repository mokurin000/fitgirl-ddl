use std::error::Error;

use compio::runtime::spawn_blocking;
use fitgirl_ddl_lib::NYQUEST_CLIENT;
use nyquest_preset::nyquest::r#async::Request;
use scraper::{Html, Selector};

#[derive(Debug, PartialEq, Eq)]
pub struct SearchEntry {
    pub title: String,
    pub href: String,
    pub date: String,
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

        let article_selector = Selector::parse("article.post").unwrap();
        let date_selector = Selector::parse("span.entry-date > a > time").unwrap();
        let title_selector = Selector::parse("h1.entry-title > a").unwrap();

        let article = html.select(&article_selector);
        article
            .map(|article| {
                let date_tag = article.select(&date_selector).next().unwrap();
                let a_tag = article.select(&title_selector).next().unwrap();

                let title = a_tag.text().collect();
                let href = a_tag.attr("href").unwrap().to_string();
                let date = date_tag
                    .attr("datetime")
                    .unwrap_or("1970-01-01T00:00:00+00:00")
                    .to_string();
                SearchEntry { title, href, date }
            })
            .filter(|SearchEntry { title, .. }| !title.starts_with("Updates Digest"))
            .collect()
    })
    .await
    .unwrap())
}
