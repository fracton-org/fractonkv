// use xxhash_rust::xxh3::xxh3_64;

// #[derive(Clone, Copy)]
// pub struct ModHasher {
//     shards: usize,
// }

// impl ModHasher {
//     pub fn new(shards: usize) -> Self {
//         assert!(shards > 0);
//         Self { shards }
//     }
//     #[inline]
//     pub fn hash(&self, key: &str) -> usize {
//         (xxh3_64(extract_hashtag(key).as_bytes()) % self.shards as u64) as usize
//     }
// }

// /// Extracts a Redis-style hashtag:
// /// - If key contains `{...}`, return the content inside the first balanced braces.
// /// - Otherwise, return the whole key.
// ///
// /// Examples:
// /// - "user:{123}"   -> "123"
// /// - "foo"          -> "foo"
// /// - "{bar}"        -> "bar"
// /// - "baz{qux}zzz"  -> "qux"
// /// - "empty:{}"     -> "empty:{}"   (ignored, empty tag)
// /// - "weird{nest{}}" -> "weird{nest{}}" (ignored, malformed)
// #[inline]
// fn extract_hashtag(key: &str) -> &str {
//     let mut chars = key.char_indices();

//     // Find the first '{'
//     if let Some((start, _)) = chars.find(|&(_, c)| c == '{') {
//         // Look for the next '}'
//         if let Some((end, _)) = key[start + 1..].char_indices().find(|&(_, c)| c == '}') {
//             let open = start + 1;
//             let close = start + 1 + end;
//             // Only use non-empty tags
//             if close > open {
//                 return &key[open..close];
//             }
//         }
//     }
//     // Fallback: no valid hashtag
//     key
// }

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use twox_hash::XxHash64;

#[derive(Clone)]
pub struct ConsistentHashRing {
    ring: BTreeMap<u64, usize>,
    vnodes: usize,
}

impl ConsistentHashRing {
    pub fn new(shard_ids: Vec<usize>, vnodes: usize) -> Self {
        let mut ring = BTreeMap::new();

        for id in shard_ids.iter().copied() {
            for vnode_id in 0..vnodes {
                let vnode_key = format!("{}-{}", id, vnode_id);
                let hash = Self::hash(&vnode_key);
                ring.insert(hash, id);
            }
        }

        Self { ring, vnodes }
    }

    pub fn get_shard<K: Hash>(&self, key: &K) -> usize {
        let hash = Self::hash(key);
        // find first vnode ≥ hash
        match self.ring.range(hash..).next() {
            Some((_, &shard_id)) => shard_id,
            None => {
                // wrap around
                *self.ring.values().next().unwrap()
            }
        }
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = XxHash64::with_seed(0); // can choose a seed
        t.hash(&mut hasher);
        hasher.finish()
    }
}
