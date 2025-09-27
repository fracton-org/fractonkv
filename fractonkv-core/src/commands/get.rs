use redis_protocol::resp3::types::BytesFrame;
use std::time::Instant;

use crate::{errors::CommandExecutionError, shard::types::DataStore};

/// Retrieve the value for a key from the data store, honoring TTL and updating access time.
///
/// If the first argument is not a simple-string key, this returns a `WrongArity("GET")` error.
/// If the key is present and its TTL (if any) has not expired, the stored data is returned as a `BytesFrame`.
/// If the key is absent or its TTL has expired, `BytesFrame::Null` is returned.
///
/// # Examples
///
/// ```
/// // Construct a `BytesFrame::SimpleString` key and a mutable `DataStore`, then call:
/// // let args = [BytesFrame::SimpleString { data: b"mykey".to_vec(), ..Default::default() }];
/// // let mut db = DataStore::default();
/// // let res = handle_get(&args, &mut db)?;
/// // assert!(matches!(res, BytesFrame::Null | BytesFrame::BulkString { .. }));
/// # Ok::<(), ()>(())
/// ```
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
