use http::header::{CONTENT_TYPE, COOKIE, ORIGIN};
use http::{HeaderValue, Method, Uri};
use tracing::debug;

use crate::errors::ExtractError;
use crate::{FUCKINGFAST_COOKIES, HTTP_CLIENT};

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

    let Some(cookies) = FUCKINGFAST_COOKIES.get() else {
        return Err(ExtractError::CookiesMissing);
    };

    // Step 1: GET request to check file status
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
        .insert(ORIGIN, HeaderValue::from_static("https://fuckingfast.co"));
    req.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    req.headers_mut().insert(COOKIE, cookies.clone());

    let post_resp = HTTP_CLIENT
        .execute(req)
        .await
        .map_err(|e| ExtractError::RequestError(e.to_string()))?;

    let hx_redirect = post_resp.headers().get("HX-Redirect").cloned();
    debug!("Response: {:?}", post_resp.bytes().await);

    // Step 4: Read HX-Redirect header
    let direct_link = hx_redirect
        .ok_or(ExtractError::DDLMissing)?
        .to_str()
        .map_err(|_| ExtractError::DDLMissing)?
        .to_string();

    Ok(DDL {
        filename,
        direct_link,
    })
}
