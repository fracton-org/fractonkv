use crate::errors::CommandExecutionError;
use crate::shard::types::DataStore;
use redis_protocol::resp3::types::BytesFrame;

/// Handle a Redis-like SET command represented by `BytesFrame` arguments and apply it to `DataStore`.
///
/// The function expects `args` to encode the SET command (key, value, and optional flags). In the
/// current implementation it does not parse or apply any flags nor modify the database; it always
/// returns a SimpleString `"OK"`.
///
/// # Examples
///
/// ```
/// let args = &[
///     BytesFrame::SimpleString { data: "mykey".into(), attributes: None },
///     BytesFrame::SimpleString { data: "myval".into(), attributes: None },
/// ];
/// let mut db = DataStore::new();
/// let res = handle_set(args, &mut db).unwrap();
/// match res {
///     BytesFrame::SimpleString { data, .. } => assert_eq!(data.as_ref(), "OK"),
///     _ => panic!("unexpected frame"),
/// }
/// ```
pub fn handle_set(
    args: &[BytesFrame],
    db: &mut DataStore,
) -> Result<BytesFrame, CommandExecutionError> {
    // match args {
    //     [key_frame, value_frame, rest @ ..] => {
    //         if rest.len() % 2 != 0 {
    //             return Err(CommandExecutionError::WrongArity("SET"));
    //         }
    //
    //         let key = match key_frame {
    //             BytesFrame::SimpleString { data, .. } => data.as_ref(),
    //             _ => return Err(CommandExecutionError::InvalidParams("Invalid key type")),
    //         };
    //
    //         let value = match value_frame {
    //             BytesFrame::SimpleString { data, .. } => data.as_ref(),
    //             _ => {
    //                 return Err(CommandExecutionError::InvalidParams("Invalid value type"));
    //             }
    //         };
    //
    //         // Parse flags into pairs
    //         let flags: Vec<(&[u8], &[u8])> = rest
    //             .chunks_exact(2)
    //             .filter_map(|chunk| match chunk {
    //                 [
    //                     BytesFrame::SimpleString { data: flag, .. },
    //                     BytesFrame::SimpleString { data: val, .. },
    //                 ] => Some((flag.as_ref(), val.as_ref())),
    //                 _ => None,
    //             })
    //             .collect();
    //
    //         let mut ttl: Option<u64> = None;
    //         let mut existence_check: Option<bool> = None;
    //         let mut get_old = false;
    //
    //         for (flag, val) in flags {
    //             match flag {
    //                 b"EX" | b"PX" => {
    //                     if ttl.is_some() {
    //                         return Err(CommandExecutionError::InvalidParams(
    //                             "Only one TTL option allowed",
    //                         ));
    //                     }
    //
    //                     let parsed: u64 = std::str::from_utf8(val)
    //                         .map_err(|_| CommandExecutionError::InvalidParams("Invalid TTL value"))?
    //                         .parse()
    //                         .map_err(|_| {
    //                             CommandExecutionError::InvalidParams("Invalid TTL value")
    //                         })?;
    //
    //                     ttl = Some(if *flag == *b"PX" {
    //                         parsed / 1000
    //                     } else {
    //                         parsed
    //                     });
    //                 }
    //
    //                 b"NX" => {
    //                     if existence_check.is_some() {
    //                         return Err(CommandExecutionError::InvalidParams(
    //                             "NX and XX cannot coexist",
    //                         ));
    //                     }
    //                     existence_check = Some(true);
    //                 }
    //
    //                 b"XX" => {
    //                     if existence_check.is_some() {
    //                         return Err(CommandExecutionError::InvalidParams(
    //                             "NX and XX cannot coexist",
    //                         ));
    //                     }
    //                     existence_check = Some(false);
    //                 }
    //
    //                 b"GET" => get_old = true,
    //                 b"KEEPTTL" => { /* optional: preserve TTL */ }
    //
    //                 _ => {
    //                     return Err(CommandExecutionError::InvalidParams("Unknown option"));
    //                 }
    //             }
    //         }
    //
    //         // Fetch previous value if GET
    //         let old_value = if get_old { db.get(key) } else { None };
    //
    //         // Respect NX/XX
    //         if let Some(nx) = existence_check {
    //             if nx && db.contains_key(key) {
    //                 return Ok(old_value
    //                     .map(|v| BytesFrame::SimpleString { data: v.into(), attributes: None })
    //                     .unwrap_or(BytesFrame::Null));
    //             }
    //             if !nx && !db.contains_key(key) {
    //                 return Ok(BytesFrame::Null);
    //             }
    //         }
    //
    //         // Insert/update value
    //         db.insert(key.to_vec().into(), value.to_vec());
    //
    //         // Set TTL if any (assuming you have a TTL map)
    //         if let Some(ttl_secs) = ttl {
    //             ttl_map.insert(key.to_vec(), ttl_secs);
    //         }
    //
    //         // Return GET old value or OK
    //         if get_old {
    //             Ok(old_value
    //                 .map(|v| BytesFrame::SimpleString { data: v.into(), attributes: None })
    //                 .unwrap_or(BytesFrame::Null))
    //         } else {
    //             Ok(BytesFrame::SimpleString { data: "OK".into(), attributes: None })
    //         }
    //     }
    //     _ => return Err(CommandExecutionError::WrongArity("SET")),
    // }
    Ok(BytesFrame::SimpleString { data: "OK".into(), attributes: None })
}
