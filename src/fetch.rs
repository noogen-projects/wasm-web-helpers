pub use reqwasm::http::{Request, Response};
pub use wasm_bindgen::JsValue;

use serde::de::{Deserialize, DeserializeOwned, Deserializer};

use crate::error::{Error, ReqwasmResult, Result};

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
            let result = fetch_success_json::<Body>(request).await;
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
        body: impl Into<JsValue>,
        callback: impl FnOnce(Result<(Response, Result<Body>)>) + 'static,
    ) {
        let request = Request::post(uri.as_ref()).body(body);
        Self::fetch(request, callback);
    }

    pub fn send_post_json<Body: 'static + DeserializeOwned>(
        uri: impl AsRef<str>,
        body: impl Into<JsValue>,
        callback: impl FnOnce(Result<(Response, Result<Body>)>) + 'static,
    ) {
        let request = Request::post(uri.as_ref())
            .header("Content-Type", "application/json")
            .body(body);
        Self::fetch(request, callback);
    }
}

pub async fn fetch(request: Request) -> ReqwasmResult<Response> {
    request.send().await
}

pub async fn fetch_text(request: Request) -> ReqwasmResult<(Response, ReqwasmResult<String>)> {
    let response = request.send().await?;
    let body = response.text().await;
    Ok((response, body))
}

pub async fn fetch_json<Body: DeserializeOwned>(request: Request) -> ReqwasmResult<(Response, ReqwasmResult<Body>)> {
    let response = request.send().await?;
    let body = response.json().await;
    Ok((response, body))
}

pub async fn fetch_success(request: Request) -> Result<Response> {
    let response = request.send().await?;
    let status = response.status();

    if status == 200 {
        Ok(response)
    } else {
        Err(Error::FailureResponse(status, format!("{:?}", response.text().await)))
    }
}

pub async fn fetch_success_text(request: Request) -> Result<(Response, Result<String>)> {
    let response = request.send().await?;
    let body = response.text().await.map_err(Into::into);
    let status = response.status();

    if status == 200 {
        Ok((response, body))
    } else {
        Err(Error::FailureResponse(status, format!("{:?}", body)))
    }
}

pub async fn fetch_success_json<Body: DeserializeOwned>(request: Request) -> Result<(Response, Result<Body>)> {
    let response = request.send().await?;
    let status = response.status();

    if status == 200 {
        let body = response.json().await.map_err(Into::into);
        Ok((response, body))
    } else {
        Err(Error::FailureResponse(status, format!("{:?}", response.text().await)))
    }
}
