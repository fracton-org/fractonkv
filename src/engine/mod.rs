mod router;

use bytes::{Buf, BytesMut};
use redis_protocol::resp2::decode::decode;
use redis_protocol::resp2::encode::encode;
use redis_protocol::resp2::types::{OwnedFrame, Resp2Frame};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;

const READ_BUF_INIT: usize = 4096;
const MAX_REQ_BYTES: usize = 4 * 1024 * 1024; // 4MB hard cap per connection
const IDLE_READ_TIMEOUT: Duration = Duration::from_secs(300); // 5 min
pub trait ServerEngine {
    fn new() -> Self;
    fn start() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}

pub struct TcpEngine;

impl ServerEngine for TcpEngine {
    fn new() -> Self {
        Self {}
    }

    fn start() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(async move {
            let listener = TcpListener::bind("0.0.0.0:6380")
                .await
                .expect("Failed to bind");

            loop {
                let (mut stream, addr) = listener.accept().await.expect("accept failed");
                // low-latency small writes
                let _ = stream.set_nodelay(true);

                tokio::spawn(async move {
                    println!("Connection from {}", addr);

                    let mut buf = BytesMut::with_capacity(READ_BUF_INIT);

                    loop {
                        // Ensure we have space to read more
                        if buf.capacity() - buf.len() < 1024 {
                            buf.reserve(READ_BUF_INIT);
                        }

                        // Bound the total buffer to avoid OOM on malicious clients
                        if buf.len() >= MAX_REQ_BYTES {
                            let _ = stream.write_all(b"-ERR request too large\r\n").await;
                            let _ = stream.shutdown().await;
                            break;
                        }

                        // Read with idle timeout
                        // let read_res = timeout(IDLE_READ_TIMEOUT, stream.read_buf(&mut buf)).await;
                        // let n = match read_res {
                        //     Err(_) => {
                        //         // idle
                        //         let _ = stream.write_all(b"-ERR idle timeout\r\n").await;
                        //         let _ = stream.shutdown().await;
                        //         break;
                        //     }
                        //     Ok(Ok(0)) => {
                        //         // peer closed
                        //         let _ = stream.shutdown().await;
                        //         break;
                        //     }
                        //     Ok(Ok(n)) => n,
                        //     Ok(Err(e)) => {
                        //         eprintln!("read error from {}: {:?}", addr, e);
                        //         let _ = stream.shutdown().await;
                        //         break;
                        //     }
                        // };

                        match timeout(IDLE_READ_TIMEOUT, stream.read_buf(&mut buf)).await {
                            Ok(read_res) => match read_res {
                                Ok(n) => {
                                    if n == 0 {
                                        let _ = stream.shutdown().await;
                                        break;
                                    }
                                    n
                                }
                                Err(err) => {}
                            },
                            Err(err) => {}
                        }

                        // Try to decode as many frames as possible from the buffer
                        let mut offset = 0usize;

                        while offset < buf.len() {
                            match decode(&buf[offset..]) {
                                Ok(Some((frame, consumed))) => {
                                    offset += consumed;

                                    // Handle the decoded frame
                                    if let Err(e) = handle_frame(&mut stream, frame).await {
                                        eprintln!("write/handle error {}: {:?}", addr, e);
                                        let _ = stream.shutdown().await;
                                        return;
                                    }
                                }
                                // Not enough data yet — wait for next read
                                Err(_) => break,
                                Ok(None) => todo!(),
                            }
                        }

                        // Discard processed bytes, keep any partial frame
                        if offset > 0 {
                            buf.advance(offset);
                        }

                        // If we read bytes but couldn't parse anything and buffer keeps growing,
                        // the cap above will eventually protect us.
                        // Otherwise we'll loop and read more until a full frame arrives.
                        let _ = n; // just to emphasize read happened
                    }
                });
            }
        })
    }
}

// Minimal RESP2 handler to demonstrate encode/decode plumbing.
// Replace with your router logic later.
async fn handle_frame(
    stream: &mut tokio::net::TcpStream,
    frame: OwnedFrame,
) -> std::io::Result<()> {
    // Redis commands typically arrive as an Array of BulkStrings
    // e.g., ["PING"], ["GET","key"], ["SET","key","value"]
    let reply = match frame {
        OwnedFrame::Array(items) => {
            let cmd = items
                .get(0)
                .and_then(|f| match f {
                    OwnedFrame::BulkString(bs) => String::from_utf8(bs.clone()).ok(),
                    OwnedFrame::SimpleString(s) => Some(String::from_utf8_lossy(s).to_string()),
                    _ => None,
                })
                .unwrap_or_default();

            match cmd.to_ascii_uppercase().as_str() {
                "PING" => OwnedFrame::SimpleString("PONG".into()),
                _ => OwnedFrame::Error("ERR unknown command".into()),
            }
        }
        _ => OwnedFrame::Error("ERR invalid frame".into()),
    };

    // Encode and write the reply
    let mut out = vec![0u8; reply.encode_len(false)];
    encode(&mut out, &reply, false).expect("encode failed");
    stream.write_all(&out).await
}
