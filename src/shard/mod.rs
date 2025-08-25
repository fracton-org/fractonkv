pub(crate) mod manager;

use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use ulid::Ulid;

use crate::store::entry::Entry;

pub type JobReceiver = Receiver<ShardJob>;
pub type ResponseSender = Sender<String>;

pub struct ShardJob {
    pub id: Ulid,
    pub entry: Entry,
    pub response: ResponseSender,
}

pub struct Shard {
    id: u8,
    db: HashMap<String, Entry>,
    jobs: JobReceiver,
}

impl Shard {
    pub fn new(id: u8, jobs: JobReceiver) -> Self {
        Self {
            id,
            db: HashMap::new(),
            jobs,
        }
    }

    pub async fn run(mut self) {
        match self.jobs.recv().await {
            Some(job) => {
                let result = self.handle_job(job).await;
                // Send the result back via oneshot channel
                if let Err(_) = result {
                    eprintln!("Shard {} failed to respond to client", self.id);
                }
            }
            // Indicates that the channel has been closes and no value will ever be received by it now
            None => {
                println!("Shard {} shutting down", self.id);
            }
        }
    }

    async fn handle_job(&mut self, job: ShardJob) -> Result<(), ()> {
        // Example: simple insert/update
        self.db.insert(job.entry.key.clone(), job.entry.clone());

        // Respond with success message
        job.response
            .send(format!(
                "OK: Shard {} stored key {}",
                self.id, job.entry.key
            ))
            .map_err(|_| ())
    }
}
