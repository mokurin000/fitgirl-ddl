# Fetch the actual direct download URL

The download flow consists of two steps performed within the same HTTP session.

## 1. Bypass Cloudflare

Issue a `GET` request to the scraped page URL and complete the Cloudflare challenge if required. Reuse the same cookies/session for the next request.

## 2. Request the download endpoint

Send a `POST` request to:

```
https://fuckingfast.co/f/<file-id>/go
```

Required request headers:

| Header           | Value                                      |
| ---------------- | ------------------------------------------ |
| `HX-Request`     | `true`                                     |
| `HX-Current-URL` | Original page URL (including the fragment) |
| `Origin`         | `https://fuckingfast.co`                   |
| `Content-Type`   | `application/x-www-form-urlencoded`        |

No request body is required.

## 3. Read the redirect URL

The response does **not** contain the download URL in the body.

Instead, read the `HX-Redirect` response header:

```
HX-Redirect: https://dl.fuckingfast.co/dl/...
```

The value of this header is the actual direct download URL and can be used to download the file immediately.

> **Note**
>
> Both requests must use the same HTTP session (cookies) established during the initial page request.
