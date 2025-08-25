use crate::engine::{ServerEngine, TcpEngine};

mod engine;
mod shard;
mod store;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    TcpEngine::start().await;
}
