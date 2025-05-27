use scraper::Selector;

use crate::{NYQUEST_CLIENT, errors::ExtractError};

#[derive(Debug, PartialEq, Eq)]
pub struct DDL {
    pub filename: String,
    pub direct_link: String,
}

pub async fn extract_ddl(url: impl AsRef<str>) -> Result<DDL, ExtractError> {
    let url = url.as_ref();

    let filename = url
        .split('#')
        .nth(1)
        .ok_or(ExtractError::FilenameMissing)?
        .to_string();

    let resp = NYQUEST_CLIENT
        .get()
        .unwrap()
        .request(nyquest::Request::get(url.to_string()))
        .await?
        .text()
        .await?;

    if resp.contains("rate limit") {
        return Err(ExtractError::RateLimited);
    }

    if resp.contains("File Not Found Or Deleted") {
        return Err(ExtractError::FileNotFound(filename));
    }

    let direct_link = compio::runtime::spawn_blocking(move || parse_html(resp))
        .await
        .map_err(|_| ExtractError::JoinError)??;

    Ok(DDL {
        filename,
        direct_link,
    })
}

fn parse_html(document: impl AsRef<str>) -> Result<String, ExtractError> {
    let document = document.as_ref();
    let document = scraper::Html::parse_document(document);
    let selector = Selector::parse("div.mx-auto > script")?;

    let script_tag = document
        .select(&selector)
        .next()
        .ok_or(ExtractError::DDLMissing)?;

    let script = script_tag.text().next().ok_or(ExtractError::DDLMissing)?;

    let (_, latter) = script
        .split_once("window.open(\"")
        .ok_or(ExtractError::DDLMissing)?;

    Ok(latter
        .split_once("\"")
        .ok_or(ExtractError::DDLMissing)?
        .0
        .to_string())
}
