use crate::shard::hasher::ConsistentHashRing;
use crate::shard::shard::Shard;
use crate::shard::types::ShardJob;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

pub struct ShardManager {
    pub num_shards: usize,
    pub senders: Vec<Sender<ShardJob>>,
}

impl ShardManager {
    /// Creates a ShardManager configured for a specific number of shards and reserves space for shard senders.
    ///
    /// # Examples
    ///
    /// ```
    /// let mgr = fractonkv_core::shard::manager::ShardManager::new(4);
    /// assert_eq!(mgr.num_shards, 4);
    /// assert!(mgr.senders.capacity() >= 4);
    /// ```
    pub fn new(num_shards: usize) -> Self {
        Self {
            num_shards,
            senders: Vec::with_capacity(num_shards),
        }
    }

    /// Start shard workers and populate the manager's mailbox senders.
    ///
    /// Creates a mailbox (Tokio mpsc channel) for each shard, stores each sender in
    /// `self.senders`, spawns one OS thread per shard running `Shard::run()`, and
    /// returns the join handles for the spawned threads.
    ///
    /// # Returns
    ///
    /// A `Vec<std::thread::JoinHandle<()>>` containing a join handle for each spawned shard thread.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut mgr = ShardManager::new(2);
    /// let handles = mgr.start();
    /// for h in handles {
    ///     let _ = h.join();
    /// }
    /// ```
    pub fn start(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        let mut receivers = Vec::with_capacity(self.num_shards);

        let _consistent_hasher = ConsistentHashRing::new((0..=self.num_shards).collect(), 64);

        self.senders = Vec::with_capacity(self.num_shards);

        // Step 1. Create mailboxes
        for _ in 0..self.num_shards {
            let (tx, rx) = channel::<ShardJob>(1024);
            self.senders.push(tx);
            receivers.push(rx);
        }

        // Step 2. Spawn shards
        let mut handles = Vec::with_capacity(self.num_shards);

        for (i, _rx) in receivers.into_iter().enumerate() {
            // let peers = self.senders.clone();

            let shard = Shard::new(i);
            // let consistent_hasher = consistent_hasher.clone();

            let handle = std::thread::spawn(move || shard.run());
            handles.push(handle);
        }
        handles
    }
}
