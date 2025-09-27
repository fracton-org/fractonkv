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
    /// Create a new `Shard` with the given identifier.
    ///
    /// The shard's in-memory `DataStore` is initialized empty.
    ///
    /// # Parameters
    ///
    /// - `id`: Shard identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// let shard = Shard::new(1);
    /// assert_eq!(shard.id, 1);
    /// assert!(shard.db.borrow().is_empty());
    /// ```
    pub fn new(id: usize) -> Self {
        Self { id, db: RefCell::new(HashMap::new()) }
    }

    /// Starts a current-thread Tokio runtime for this shard, binds to 127.0.0.1:6380, and runs the shard's listener loop.
    ///
    /// This method consumes the shard, initializes a single-threaded Tokio runtime and LocalSet, and blocks the current thread while the shard accepts and handles TCP connections on 127.0.0.1:6380.
    ///
    /// # Examples
    ///
    /// ```
    /// let shard = Shard::new(0);
    /// // This call blocks the current thread and begins accepting connections on 127.0.0.1:6380.
    /// shard.run();
    /// ```
    pub fn run(self) {
        let local = tokio::task::LocalSet::new();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(local.run_until(async move {
            self.bind_and_listen("127.0.0.1:6380").await;
        }));
    }

    /// Bind a TCP listener to `addr` and accept incoming connections, handling each connection concurrently.
    ///
    /// This method creates a TCP listener on the provided address, logs the listening address, and enters
    /// an accept loop. Each accepted connection is handled by spawning the shard's per-connection handler
    /// into an internal concurrent task set so multiple connections are processed simultaneously. The loop
    /// runs until the surrounding task is cancelled or the process exits.
    ///
    /// # Panics
    ///
    /// Panics if binding to `addr` fails or if accepting a connection fails (calls to `unwrap()` inside the loop).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use fractonkv_core::shard::Shard;
    /// # async fn run_example() {
    /// let shard = Shard::new(0);
    /// // Start the listener in the background and then abort so the example does not hang.
    /// let handle = tokio::spawn(async move { shard.bind_and_listen("127.0.0.1:0").await });
    /// handle.abort();
    /// # }
    /// ```
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

    /// Handles a single TCP connection: reads RESP3 frames, processes each frame against the shard's
    /// data store, and sends back the resulting RESP3 frames until the connection closes or an I/O
    /// error occurs.
    ///
    /// The connection is processed in a loop: for each inbound frame a mutable borrow of the shard's
    /// in-memory `DataStore` is taken only for the duration of handling that frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::net::{TcpListener, TcpStream};
    /// use tokio::task;
    ///
    /// #[tokio::test]
    /// async fn example_handle_connection() {
    ///     // Start a listener and connect a client to it to obtain a TcpStream pair.
    ///     let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    ///     let addr = listener.local_addr().unwrap();
    ///
    ///     let client = task::spawn(async move {
    ///         TcpStream::connect(addr).await.unwrap()
    ///     });
    ///
    ///     let (server_stream, peer_addr) = listener.accept().await.unwrap();
    ///
    ///     // Construct a shard and drive the connection handler (runs until the client disconnects).
    ///     let shard = crate::shard::Shard::new(0);
    ///     task::spawn(async move {
    ///         shard.handle_connection(server_stream, peer_addr).await;
    ///     });
    ///
    ///     // Drop the client to terminate the example connection.
    ///     drop(client.await.unwrap());
    /// }
    /// ```
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

/// Validates an incoming RESP3 frame and produces a response frame after attempting command dispatch.
///
/// If the provided frame is not a non-empty Array, returns a `SimpleError` frame with the message `"ERR invalid frame"`.
/// For array frames this function parses the command but currently returns a `SimpleError` frame with the message
/// `"ERR command not implemented"`.
///
/// # Returns
///
/// A `BytesFrame` representing the response: an error frame for invalid frames or for commands that are not implemented.
///
/// # Examples
///
/// ```no_run
/// // Non-array frame -> invalid frame error
/// let frame = BytesFrame::SimpleError { data: "err".into(), attributes: None };
/// let mut db = /* DataStore instance */;
/// let resp = handle_frame(&frame, &mut db);
/// if let BytesFrame::SimpleError { data, .. } = resp {
///     assert_eq!(data, "ERR invalid frame");
/// }
/// ```
fn handle_frame(frame: &BytesFrame, db: &mut DataStore) -> BytesFrame {
    let arr = match frame {
        BytesFrame::Array { data, .. } if !data.is_empty() => data,
        _ => {
            return BytesFrame::SimpleError { data: "ERR invalid frame".into(), attributes: None };
        }
    };

    let args: &[BytesFrame] = &arr[1..];
    let cmd = CommandKind::from_frame(frame).unwrap();

    BytesFrame::SimpleError {
        data: "ERR command not implemented".into(),
        attributes: None,
    }
}
