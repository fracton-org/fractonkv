use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc::{self, Sender as MpscSender};
use tokio::sync::oneshot;
use ulid::Ulid;
use crate::shard::{Shard, ShardJob};
use crate::store::entry::Entry;


pub struct ShardManager {
    shard_senders: Vec<MpscSender<ShardJob>>,
}

impl ShardManager {
    pub fn new(num_shards: usize) -> Self {
        let mut shard_senders = Vec::with_capacity(num_shards);

        for id in 0..num_shards {
            let (tx, rx) = mpsc::channel(100);
            let shard = Shard::new(id as u8, rx);

            tokio::spawn(shard.run());
            shard_senders.push(tx);
        }
        Self { shard_senders }
    }

    fn shard_for_key(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_senders.len()
    }

    pub async fn dispatch(&self, entry: Entry) -> Result<String, ()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let job = ShardJob {
            id: Ulid::new(),
            entry,
            response: resp_tx,
        };

        let shard_idx = self.shard_for_key(&job.entry.key);
        self.shard_senders[shard_idx]
            .send(job)
            .await  
            .map_err(|_| ())?;

        resp_rx.await.map_err(|_| ())
    }
}
