use futures_util::TryFutureExt;
use scraper::Selector;
use url::Url;

use crate::{NYQUEST_CLIENT, errors::ScrapeError};

#[derive(Debug)]
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
        .await?
        .text()
        .await?;

    let fuckingfast_links = compio::runtime::spawn_blocking(move || parse_html(resp))
        .map_err(|_| ScrapeError::JoinError)
        .await??;

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

    let single_tag;

    match tags.len() {
        0 => return Err(ScrapeError::FuckingFastSourceMissing)?,
        1 => {
            single_tag = tags[0];
        }
        _ => return Err(ScrapeError::UnexpectedURL)?,
    }

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
