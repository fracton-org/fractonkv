use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::time::Instant;

/// Metadata + the actual stored value.
#[derive(Debug, Clone)]
pub struct Entry {
    pub key: String,
    pub value: DataKind,
    pub ttl: Option<Instant>, // expiration time
    pub last_accessed: Option<Instant>,
    pub created_at: Instant, // for bookkeeping
}

#[derive(Debug, Clone)]
pub enum DataKind {
    String(String),
    List(VecDeque<String>),
    Set(HashSet<String>),
    Hash(HashMap<String, String>),
    SortedSet(BTreeMap<String, f64>),
}

impl Display for DataKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataKind::String(str) => {
                write!(f, "{}", str)
            }
            DataKind::List(list) => {
                for (i, item) in list.iter().enumerate() {
                    writeln!(f, "{}) {}", i + 1, item)?;
                }
                Ok(())
            }
            DataKind::Set(set) => {
                for (i, item) in set.iter().enumerate() {
                    writeln!(f, "{}) {}", i + 1, item)?;
                }
                Ok(())
            }
            DataKind::Hash(hash) => {
                for (i, (k, v)) in hash.iter().enumerate() {
                    writeln!(f, "{}) {}: {}", i + 1, k, v)?;
                }
                Ok(())
            }
            DataKind::SortedSet(sset) => {
                for (i, (member, score)) in sset.iter().enumerate() {
                    writeln!(f, "{}) {} ({})", i + 1, member, score)?;
                }
                Ok(())
            }
        }
    }
}
