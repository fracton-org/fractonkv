use redis_protocol::resp3::types::BytesFrame;
use std::time::Instant;

use crate::{errors::CommandExecutionError, shard::types::DataStore};

pub fn handle_get(
    args: &[BytesFrame],
    db: &mut DataStore,
) -> Result<BytesFrame, CommandExecutionError> {
    let key = match args.first() {
        Some(BytesFrame::SimpleString { data, .. }) => data,
        _ => return Err(CommandExecutionError::WrongArity("GET")),
    };

    match db.get_mut(key) {
        Some(v) => {
            if let Some(ttl) = v.ttl
                && Instant::now() >= v.created_at + ttl
            {
                return Ok(BytesFrame::Null);
            }
            v.last_accessed = Instant::now();
            Ok(v.data.to_bytes_frame())
        }
        None => Ok(BytesFrame::Null),
    }
}
