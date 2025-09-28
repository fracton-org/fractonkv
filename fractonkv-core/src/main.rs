use crate::shard::manager::ShardManager;
use tracing::Level;

mod commands;
mod errors;
mod shard;

fn main() {
    tracing_subscriber::fmt().with_target(false).with_max_level(Level::DEBUG).init();

    print_banner();

    let num_shards = num_cpus::get();
    let mut shard_manager = ShardManager::new(num_shards);
    let handles = shard_manager.start();

    // main decides to block until threads finish
    for handle in handles {
        handle.join().unwrap();
    }
}

fn print_banner() {
    println!(
        r#"
                                              ██████████
                                              ██████████
                                              ██████████
                                              ██████████
                       ██████████             ██████████              █████████
                       ██████████                 ██                  █████████
                       ██████████                 ██                  █████████
                       ██████████                 ██                  █████████
                       ██████████                 ██                  █████████
                                 ███              ██              ███
                                   ███            ██            ███
                                     ███          ██          ███
                                       ███        ██        ███
                                         ███      ██      ███
                                           ███    ██    ███
                       █████████             ███  ██  ███               █████████
                       █████████               ████████                 █████████
                       ██████████████████████████████████████████████████████████
                       █████████                  ██                    █████████
                       █████████                  ██                    █████████
                                                  ██
                                                  ██
                                                  ██
                                                  ██
                                                  ██
                                                  ██
    "#
    );
}
