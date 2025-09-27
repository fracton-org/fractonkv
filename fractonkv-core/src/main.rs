use crate::shard::manager::ShardManager;
use tracing::Level;

mod commands;
mod errors;
mod shard;

/// Initializes logging, prints the ASCII banner, spawns shard workers equal to CPU cores, and blocks until all worker threads exit.
///
/// # Examples
///
/// ```
/// // Entry point for the binary; runs initialization and waits for shard workers to finish.
/// main();
/// ```
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

/// Prints the program's ASCII art banner to standard output.
///
/// # Examples
///
/// ```rust
/// // Display the application banner
/// print_banner();
/// ```
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
