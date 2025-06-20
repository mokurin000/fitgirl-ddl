use scraper::Selector;
use url::Url;

use crate::{NYQUEST_CLIENT, errors::ScrapeError};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub path_part: String,
    pub fuckingfast_links: Vec<String>,
}

pub async fn scrape_game(url: impl AsRef<str>) -> Result<GameInfo, ScrapeError> {
    let url = Url::parse(url.as_ref())?;

    let path_part = url
        .path_segments()
        .ok_or(ScrapeError::IllFormedURL(
            url::ParseError::RelativeUrlWithCannotBeABaseBase,
        ))?
        .next()
        // unlikely to happen on valid https URLs
        .ok_or(ScrapeError::UnexpectedURL)?
        .to_string();

    let resp = NYQUEST_CLIENT
        .get()
        .unwrap()
        .request(nyquest::Request::get(url.to_string()))
        .await
        .map_err(|e| ScrapeError::RequestError(e.to_string()))?
        .text()
        .await
        .map_err(|e| ScrapeError::RequestError(e.to_string()))?;

    #[cfg(feature = "compio")]
    let spawn = compio::runtime::spawn_blocking;
    #[cfg(feature = "tokio")]
    let spawn = tokio::task::spawn_blocking;

    let fuckingfast_links = spawn(move || parse_html(resp))
        .await
        .map_err(|_| ScrapeError::JoinError)??;

    Ok(GameInfo {
        path_part,
        fuckingfast_links,
    })
}

fn parse_html(document: impl AsRef<str>) -> Result<Vec<String>, ScrapeError> {
    let document = document.as_ref();
    let document = scraper::Html::parse_document(document);
    let selector = Selector::parse("div.entry-content > ul > li > a")?;

    let tags = document
        .select(&selector)
        .filter(|tag| {
            tag.text()
                .next()
                .is_some_and(|t| t == "Filehoster: FuckingFast")
        })
        .collect::<Vec<_>>();

    let single_tag = match tags.len() {
        0 => return Err(ScrapeError::FuckingFastSourceMissing)?,
        1 => tags[0],
        _ => return Err(ScrapeError::UnexpectedURL)?,
    };

    let fuckingfast_links_selector = Selector::parse(
        "div.entry-content > ul > li > div.su-spoiler > div.su-spoiler-content > a",
    )?;

    let fuckingfast_links: Vec<_> = document
        .select(&fuckingfast_links_selector)
        .filter_map(|tag| tag.attr("href"))
        .filter(|href| href.starts_with("https://fuckingfast.co"))
        .map(str::to_string)
        .collect();

    match fuckingfast_links.len() {
        0 => Ok(vec![
            single_tag
                .attr("href")
                .ok_or(ScrapeError::FuckingFastSourceMissing)?
                .to_string(),
        ]),
        _ => Ok(fuckingfast_links),
    }
}
