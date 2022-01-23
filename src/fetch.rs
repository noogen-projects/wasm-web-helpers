pub use reqwasm::http::{Request, Response};

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};

#[derive(Copy, Clone)]
pub struct MissingBody;

trait Missing {
    fn missing() -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<T: DeserializeOwned> Missing for T {}

impl Missing for MissingBody {
    fn missing() -> Option<Self> {
        Some(Self)
    }
}

#[derive(Default)]
pub struct JsonFetcher;

impl JsonFetcher {
    pub fn fetch<Body: 'static + DeserializeOwned>(
        request: Request,
        callback: impl FnOnce(Result<(Response, Body)>) + 'static,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            let result = fetch_json::<Body>(request).await;
            callback(result);
        });
    }

    pub fn send_get<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        callback: impl FnOnce(Result<(Response, Body)>) + 'static,
    ) {
        let request = Request::get(uri.as_ref());
        Self::fetch(request, callback);
    }

    pub fn send_post<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        body: impl Into<String>,
        callback: impl FnOnce(Result<(Response, Body)>) + 'static,
    ) {
        let request = Request::post(uri.as_ref()).body(body.into());
        Self::fetch(request, callback);
    }

    pub fn send_post_json<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        body: impl Into<String>,
        callback: impl FnOnce(Result<(Response, Body)>) + 'static,
    ) {
        let request = Request::post(uri.as_ref())
            .header("Content-Type", "application/json")
            .body(body.into());
        Self::fetch(request, callback);
    }
}

async fn fetch_json<Body: DeserializeOwned>(request: Request) -> Result<(Response, Body)> {
    let response = request.send().await?;
    if response.status() == 200 {
        let body = if let Some(body) = Body::missing() {
            body
        } else {
            response.json().await?
        };
        Ok((response, body))
    } else {
        Err(Error::FailureResponse(
            response.status(),
            format!("{:?}", response.text().await),
        ))
    }
}
