use http::{Method, Uri};
use scraper::Selector;
use tracing::debug;

use crate::HTTP_CLIENT;
use crate::errors::ExtractError;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
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
    let uri: Uri = url.parse()?;

    let resp = HTTP_CLIENT
        .request(Method::GET, uri)
        .send()
        .await
        .map_err(|e| ExtractError::RequestError(e.to_string()))?
        .text()
        .await
        .map_err(|e| ExtractError::RequestError(e.to_string()))?;

    if resp.contains("rate limit") {
        return Err(ExtractError::RateLimited);
    }

    if resp.contains("File Not Found Or Deleted") {
        return Err(ExtractError::FileNotFound(filename));
    }

    #[cfg(feature = "compio")]
    let direct_link = compio::runtime::spawn_blocking(move || parse_html(resp))
        .await
        .map_err(|_| ExtractError::JoinError)??;
    #[cfg(feature = "tokio")]
    let direct_link = tokio::task::spawn_blocking(move || parse_html(resp))
        .await
        .map_err(|_| ExtractError::JoinError)??;

    Ok(DDL {
        filename,
        direct_link,
    })
}

fn parse_html(document: impl AsRef<str>) -> Result<String, ExtractError> {
    let document = document.as_ref();
    debug!("parsing document: {document}");

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
