use crate::commands::CommandKind;
use crate::shard::types::DataStore;
use futures::FutureExt;

use futures::stream::FuturesUnordered;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use redis_protocol::codec::Resp3;
use redis_protocol::resp3::types::BytesFrame;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_util::codec::Framed;

pub struct Shard {
    pub id: usize,
    pub db: RefCell<DataStore>,
}

impl Shard {
    pub fn new(id: usize) -> Self {
        Self { id, db: RefCell::new(HashMap::new()) }
    }

    pub fn run(self) {
        let local = tokio::task::LocalSet::new();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(local.run_until(async move {
            self.bind_and_listen("127.0.0.1:6380").await;
        }));
    }

    async fn bind_and_listen(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();
        info!("Shard {} listening on {}", self.id, addr);

        let mut connections = FuturesUnordered::new();

        loop {
            tokio::select! {
                accept = listener.accept().fuse() => {
                    let (stream, peer_addr) = accept.unwrap();
                    info!("Accepted connection from {}", peer_addr);

                    // Push a new future into the unordered set
                    // pass a reference to self.db instead of &mut self
                    connections.push(self.handle_connection(stream, peer_addr));
                }
                Some(_) = connections.next() => {
                    // A connection finished; automatically polled
                }
            }
        }
    }

    async fn handle_connection(&self, stream: tokio::net::TcpStream, peer_addr: SocketAddr) {
        let mut framed = Framed::new(stream, Resp3::default());

        while let Some(result) = framed.next().await {
            match result {
                Ok(frame) => {
                    info!("Shard {} got frame from {}: {:?}", self.id, peer_addr, &frame);

                    // Borrow db mutably only for this frame
                    let response = handle_frame(&frame, &mut self.db.borrow_mut());

                    if let Err(e) = framed.send(response).await {
                        error!("Write error to {}: {}", peer_addr, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading from {}: {}", peer_addr, e);
                    break;
                }
            }
        }
        info!("Connection closed: {}", peer_addr);
    }
}

fn handle_frame(frame: &BytesFrame, db: &mut DataStore) -> BytesFrame {
    let arr = match frame {
        BytesFrame::Array { data, .. } if !data.is_empty() => data,
        _ => {
            return BytesFrame::SimpleError { data: "ERR invalid frame".into(), attributes: None };
        }
    };

    let args: &[BytesFrame] = &arr[1..];
    let cmd = match CommandKind::from_frame(frame) {
        Ok(cmd) => cmd,
        Err(err) => return BytesFrame::from(err),
    };

    BytesFrame::SimpleError {
        data: "ERR command not implemented".into(),
        attributes: None,
    }
}
