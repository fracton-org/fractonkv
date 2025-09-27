use bytes::Bytes;
use redis_protocol::resp3::types::BytesFrame;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{Duration, Instant};
use strum_macros::Display;

pub type ShardJob = String;

pub type DataStore = HashMap<Bytes, StoreObject>;
#[derive(Debug)]
pub struct StoreObject {
    pub data: DataKind,
    pub ttl: Option<Duration>,
    pub last_accessed: Instant,
    pub created_at: Instant,
}
#[derive(Debug, Display)]
pub enum DataKind {
    /// Simple text or small values
    String(Bytes),
    /// Binary-safe value
    BulkString(Bytes),
    /// Hash (field -> value)
    Hash(HashMap<Bytes, Bytes>),
    /// List of values
    List(Vec<DataKind>),
    /// Set of unique values
    Set(HashSet<Bytes>),
    /// Sorted Set (score -> member)
    SortedSet(BTreeMap<f64, Bytes>),
}

impl DataKind {
    pub fn to_bytes_frame(&self) -> BytesFrame {
        match self {
            DataKind::String(s) => {
                BytesFrame::SimpleString { data: s.clone().into(), attributes: None }
            }

            DataKind::BulkString(bytes) => {
                BytesFrame::BlobString { data: bytes.clone().into(), attributes: None }
            }

            DataKind::Hash(hash_map) => {
                let mut frames = Vec::with_capacity(hash_map.len() * 2);
                for (k, v) in hash_map {
                    frames.push(BytesFrame::SimpleString {
                        data: k.clone().into(),
                        attributes: None,
                    });
                    frames.push(BytesFrame::SimpleString {
                        data: v.clone().into(),
                        attributes: None,
                    });
                }
                BytesFrame::Array { data: frames, attributes: None }
            }

            DataKind::List(data_kinds) => {
                let mut frames = Vec::with_capacity(data_kinds.len());

                for dk in data_kinds {
                    frames.push(dk.to_bytes_frame());
                }
                BytesFrame::Array { data: frames, attributes: None }
            }

            DataKind::Set(hash_set) => {
                let mut frames = Vec::with_capacity(hash_set.len());

                for item in hash_set {
                    frames.push(BytesFrame::SimpleString {
                        data: item.clone().into(),
                        attributes: None,
                    });
                }
                BytesFrame::Array { data: frames, attributes: None }
            }

            DataKind::SortedSet(btree_map) => {
                let mut frames = Vec::with_capacity(btree_map.len() * 2);
                for (score, member) in btree_map {
                    frames.push(BytesFrame::SimpleString {
                        data: score.to_string().into(),
                        attributes: None,
                    });
                    frames.push(BytesFrame::SimpleString {
                        data: member.clone().into(),
                        attributes: None,
                    });
                }
                BytesFrame::Array { data: frames, attributes: None }
            }
        }
    }
    pub fn from_bytes_frame(frame: &BytesFrame) -> Option<Self> {
        match frame {
            BytesFrame::SimpleString { data, .. } => Some(DataKind::String(data.clone())),
            BytesFrame::BlobString { data, .. } => Some(DataKind::BulkString(data.clone())),
            BytesFrame::Array { data, .. } => {
                // Heuristic: try to guess the type of array:
                // 1. Even length & all key/value frames -> Hash or SortedSet
                // 2. Otherwise -> List
                // Note: This depends on your encoding convention

                // Try Hash: even length, all pairs are SimpleString
                if data.len() % 2 == 0
                    && data.iter().all(|f| matches!(f, BytesFrame::SimpleString { .. }))
                {
                    let mut hash_map = HashMap::with_capacity(data.len() / 2);
                    let mut iter = data.iter();
                    while let (Some(kf), Some(vf)) = (iter.next(), iter.next()) {
                        if let (
                            BytesFrame::SimpleString { data: k, .. },
                            BytesFrame::SimpleString { data: v, .. },
                        ) = (kf, vf)
                        {
                            hash_map.insert(k.clone(), v.clone());
                        } else {
                            return None;
                        }
                    }
                    return Some(DataKind::Hash(hash_map));
                }

                // Otherwise, treat as List
                let mut list = Vec::with_capacity(data.len());
                for item in data {
                    list.push(DataKind::from_bytes_frame(item)?);
                }
                Some(DataKind::List(list))
            }
            _ => None,
        }
    }
}

// impl From<DataKind> for Bytes {
//     fn from(value: DataKind) -> Self {
//         match value {
//             DataKind::String(s) => Bytes::from(s),
//             DataKind::BulkString(bs) => Bytes::from(bs),

//             DataKind::Hash(hash) => {
//                 // serialize as key=value\n
//                 let mut buf = String::new();
//                 for (k, v) in hash {
//                     buf.push_str(&k);
//                     buf.push('=');
//                     buf.push_str(&v);
//                     buf.push('\n');
//                 }
//                 Bytes::from(buf)
//             }

//             DataKind::List(list) => {
//                 // recursively convert each element to bytes and join
//                 let mut buf = Vec::new();
//                 for (i, item) in list.into_iter().enumerate() {
//                     if i > 0 {
//                         buf.extend_from_slice(b"\n");
//                     }
//                     let part: Bytes = item.into();
//                     buf.extend_from_slice(&part);
//                 }
//                 Bytes::from(buf)
//             }

//             DataKind::Set(hash_set) => {
//                 // join by newline
//                 let mut buf = String::new();
//                 for (i, val) in hash_set.into_iter().enumerate() {
//                     if i > 0 {
//                         buf.push('\n');
//                     }
//                     buf.push_str(&val);
//                 }
//                 Bytes::from(buf)
//             }

//             DataKind::SortedSet(btree_map) => {
//                 // format as score:member\n
//                 let mut buf = String::new();
//                 for (score, member) in btree_map {
//                     buf.push_str(&format!("{}:{}\n", score, member));
//                 }
//                 Bytes::from(buf)
//             }
//         }
//     }
// }
