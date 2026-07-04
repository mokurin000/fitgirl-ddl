use http::{HeaderValue, Method, Uri};

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

    // Step 1: GET request to bypass Cloudflare and establish session
    let resp = HTTP_CLIENT
        .request(Method::GET, uri.clone())
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

    // Step 2 & 3: Extract file ID and POST to download endpoint
    let file_id = uri.path().trim_start_matches('/');
    let post_uri: Uri = format!("https://fuckingfast.co/f/{file_id}/go").parse()?;

    let mut req = wreq::Request::new(Method::POST, post_uri);
    req.headers_mut()
        .insert("HX-Request", HeaderValue::from_static("true"));
    req.headers_mut().insert(
        "HX-Current-URL",
        HeaderValue::from_str(url).map_err(|e| ExtractError::RequestError(e.to_string()))?,
    );
    req.headers_mut()
        .insert("Origin", HeaderValue::from_static("https://fuckingfast.co"));
    req.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    let post_resp = HTTP_CLIENT
        .execute(req)
        .await
        .map_err(|e| ExtractError::RequestError(e.to_string()))?;

    // Step 4: Read HX-Redirect header
    let direct_link = post_resp
        .headers()
        .get("HX-Redirect")
        .ok_or(ExtractError::DDLMissing)?
        .to_str()
        .map_err(|_| ExtractError::DDLMissing)?
        .to_string();

    Ok(DDL {
        filename,
        direct_link,
    })
}
