pub use reqwasm::http::{Request, Response};

use serde::de::{Deserialize, DeserializeOwned, Deserializer};

use crate::error::{Error, Result};

#[derive(Copy, Clone)]
pub struct MissingBody;

impl<'de> Deserialize<'de> for MissingBody {
    fn deserialize<D>(_deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self)
    }
}

#[derive(Default)]
pub struct JsonFetcher;

impl JsonFetcher {
    pub fn fetch<Body: 'static + DeserializeOwned>(
        request: Request,
        callback: impl FnOnce(Result<(Response, Result<Body>)>) + 'static,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            let result = fetch_json::<Body>(request).await;
            callback(result);
        });
    }

    pub fn send_get<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        callback: impl FnOnce(Result<(Response, Result<Body>)>) + 'static,
    ) {
        let request = Request::get(uri.as_ref());
        Self::fetch(request, callback);
    }

    pub fn send_post<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        body: impl Into<String>,
        callback: impl FnOnce(Result<(Response, Result<Body>)>) + 'static,
    ) {
        let request = Request::post(uri.as_ref()).body(body.into());
        Self::fetch(request, callback);
    }

    pub fn send_post_json<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        body: impl Into<String>,
        callback: impl FnOnce(Result<(Response, Result<Body>)>) + 'static,
    ) {
        let request = Request::post(uri.as_ref())
            .header("Content-Type", "application/json")
            .body(body.into());
        Self::fetch(request, callback);
    }
}

async fn fetch_json<Body: DeserializeOwned>(request: Request) -> Result<(Response, Result<Body>)> {
    let response = request.send().await?;
    if response.status() == 200 {
        let body = response.json().await.map_err(Into::into);
        Ok((response, body))
    } else {
        Err(Error::FailureResponse(
            response.status(),
            format!("{:?}", response.text().await),
        ))
    }
}
