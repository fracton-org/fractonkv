mod codec;
mod router;

use crate::engine::codec::RedisCodec;
use bytes::Bytes;
use futures::{Sink, SinkExt, StreamExt};
use redis_protocol::resp2::types::{BytesFrame, Resp2Frame};
use std::{future::Future, pin::Pin};
use tokio::{
    net::TcpListener,
    time::{timeout, Duration},
};
use tokio_util::codec::Framed;

const IDLE_READ_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_REQ_BYTES: usize = 16 * 1024;

pub struct TcpEngine;

impl TcpEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(async move {
            let listener = TcpListener::bind("0.0.0.0:6380")
                .await
                .expect("Failed to bind");

            loop {
                let (stream, addr) = listener.accept().await.expect("accept failed");
                let _ = stream.set_nodelay(true);
                tokio::spawn(handle_connection(stream, addr));
            }
        })
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, addr: std::net::SocketAddr) {
    println!("Connection from {}", addr);

    let mut framed = Framed::new(stream, RedisCodec::new());

    loop {
        // Timeout around one poll of the stream
        let next = match timeout(IDLE_READ_TIMEOUT, framed.next()).await {
            Ok(opt) => opt, // Option<Result<OwnedFrame, io::Error>>
            Err(_) => {
                // idle timeout
                let _ = framed.send(BytesFrame::Error("idle timeout".into())).await;
                break;
            }
        };

        // peer closed
        let Some(res) = next else { break };

        let frame = match res {
            Ok(f) => f,
            Err(e) => {
                eprintln!("read/parse error from {}: {:?}", addr, e);
                break;
            }
        };

        if let Err(e) = handle_frame(&mut framed, frame).await {
            eprintln!("handle_frame error for {}: {:?}", addr, e);
            break;
        }
    }

    let _ = framed.close().await;
}

async fn handle_frame<S>(framed: &mut S, frame: BytesFrame) -> std::io::Result<()>
where
    S: Sink<BytesFrame, Error = std::io::Error> + Unpin,
{
    framed
        .send(BytesFrame::SimpleString(Bytes::from("OK")))
        .await
}
