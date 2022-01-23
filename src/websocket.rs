pub use gloo_utils::errors::JsError;
pub use reqwasm::websocket::{futures::WebSocket, Message, WebSocketError};

use futures_channel::mpsc;
use futures_util::{stream::StreamExt, SinkExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    Break,
    Continue,
}

impl From<()> for ControlFlow {
    fn from(_: ()) -> Self {
        Self::Continue
    }
}

#[derive(Clone)]
pub struct WebSocketService {
    sender: mpsc::UnboundedSender<Message>,
}

impl WebSocketService {
    pub fn open<S, R>(
        url: impl AsRef<str>,
        send_callback: impl Fn(Result<(), WebSocketError>) -> S + 'static,
        receive_callback: impl Fn(Result<Message, WebSocketError>) -> R + 'static,
        close_send_callback: impl FnOnce() + 'static,
        close_receive_callback: impl FnOnce() + 'static,
    ) -> Result<Self, JsError>
    where
        S: Into<ControlFlow>,
        R: Into<ControlFlow>,
    {
        let ws = WebSocket::open(url.as_ref())?;
        let (mut sink, mut stream) = ws.split();
        let (sender, mut receiver) = mpsc::unbounded();

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(msg) = receiver.next().await {
                if send_callback(sink.send(msg).await).into() == ControlFlow::Break {
                    break;
                }
            }
            close_send_callback();
        });

        wasm_bindgen_futures::spawn_local(async move {
            while let Some(msg) = stream.next().await {
                if receive_callback(msg).into() == ControlFlow::Break {
                    break;
                }
            }
            close_receive_callback();
        });

        Ok(Self { sender })
    }

    pub fn send(&mut self, msg: Message) -> Result<(), mpsc::TrySendError<Message>> {
        self.sender.unbounded_send(msg)
    }
}
