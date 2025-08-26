use crate::engine::TcpEngine;

mod engine;
mod shard;
mod store;

// Usage example
#[tokio::main]
async fn main() {
    let engine = TcpEngine::new();
    engine.start().await;
}
