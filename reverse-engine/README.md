# Fetch the actual direct download URL

The download flow consists of two steps performed within the same HTTP session.

> [!NOTE]
> Both requests must use the same HTTP session (cookies) established during the initial page request.

## 1. Bypass Cloudflare

Issue a `GET` request to the scraped FuckingFast page and bypass the Cloudflare challenge via `wreq`.

## 2. Extract the file ID

The **file ID** is simply the path component of the scraped FuckingFast URL.

For example:

| Page URL                              | File ID        |
| ------------------------------------- | -------------- |
| `https://fuckingfast.co/w7sfvwatp70x` | `w7sfvwatp70x` |

## 3. Request the download endpoint

Send a `POST` request to:

```
https://fuckingfast.co/f/<file-id>/go
```

Required request headers:

| Header           | Value                                                 |
| ---------------- | ----------------------------------------------------- |
| `HX-Request`     | `true`                                                |
| `HX-Current-URL` | Original page URL (including the fragment if present) |
| `Origin`         | `https://fuckingfast.co`                              |
| `Content-Type`   | `application/x-www-form-urlencoded`                   |

No request body is required.

## 4. Read the redirect URL

The response body is empty. Instead, read the `HX-Redirect` response header:

```
HX-Redirect: https://dl.fuckingfast.co/dl/...
```

The value of this header is the actual direct download URL.
