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
    /// Constructs a consistent-hash ring and populates it with virtual nodes for each shard.
    ///
    /// Each provided shard ID receives `vnodes` virtual nodes; each virtual node is hashed
    /// and inserted into the ring mapping hash -> shard ID.
    ///
    /// # Parameters
    ///
    /// - `shard_ids`: a list of shard identifiers to include in the ring.
    /// - `vnodes`: number of virtual nodes to create for each shard (must be >= 1 to produce entries).
    ///
    /// # Examples
    ///
    /// ```
    /// let ring = ConsistentHashRing::new(vec![0usize, 1usize], 10);
    /// let shard = ring.get_shard(&"some-key");
    /// assert!(shard == 0 || shard == 1);
    /// ```
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

    /// Selects the shard responsible for a given key on the consistent hash ring.
    
    ///
    
    /// The key is hashed and the virtual node with the smallest hash greater than or
    
    /// equal to the key's hash is chosen; if the key's hash is greater than all
    
    /// virtual node hashes the selection wraps to the first shard in the ring.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```
    
    /// let ring = ConsistentHashRing::new(vec![0usize, 1usize], 2);
    
    /// let shard = ring.get_shard(&"my-key");
    
    /// assert!(shard == 0 || shard == 1);
    
    /// ```
    
    ///
    
    /// Returns the identifier of the shard responsible for `key`.
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

    /// Computes a stable 64-bit hash of the given value using the XxHash64 algorithm seeded with 0.
    ///
    /// The resulting `u64` is derived by feeding the provided value into an `XxHash64` hasher
    /// constructed with seed `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// let h1 = hash(&"example");
    /// let h2 = hash(&"example");
    /// assert_eq!(h1, h2);
    /// ```
    fn hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = XxHash64::with_seed(0); // can choose a seed
        t.hash(&mut hasher);
        hasher.finish()
    }
}
