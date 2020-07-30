use std::cmp::Ordering;
// use ::{N_BUCKETS,K_PARAM};
use super::key::{Distance, Key, KEY_LEN};
use log::{info, warn};
// use serde::{Deserialize, Serialize};

pub const N_BUCKETS: usize = KEY_LEN * 8;
pub const K_ENTRIES_PER_BUCKET: usize = 8;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Key,
    pub addr: String,
    pub net_id: String,
}

#[derive(Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct KnownNode {
    // is KnownNode even needed ? Maybe calculate the distance dynamically
    pub node: Node,
    pub distance: Distance,
}

impl PartialEq for KnownNode {
    fn eq(&self, other: &KnownNode) -> bool {
        self.distance.eq(&other.distance)
    }
}
impl PartialOrd for KnownNode {
    fn partial_cmp(&self, other: &KnownNode) -> Option<Ordering> {
        Some(other.distance.cmp(&self.distance))
    }
}
impl Ord for KnownNode {
    fn cmp(&self, other: &KnownNode) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

#[derive(Debug)]
pub struct RoutingTable {
    node_info: Node,
    buckets: Vec<Vec<Node>>,
}

impl RoutingTable {
    pub fn new(node_info: Node) -> RoutingTable {
        let mut buckets = Vec::new();
        for _ in 0..N_BUCKETS {
            buckets.push(Vec::new());
        }
        let mut ret = RoutingTable {
            node_info: node_info.clone(),
            buckets: buckets,
        };
        ret.update(node_info.clone());
        ret
    }

    /// Update the appropriate bucket with the new node's info
    pub fn update(&mut self, node_info: Node) {
        let bucket_index = self.lookup_bucket_index(node_info.id);
        let bucket = &mut self.buckets[bucket_index];
        let node_index = bucket.iter().position(|x| x.id == node_info.id);
        match node_index {
            Some(i) => {
                let temp = bucket.remove(i);
                bucket.push(temp);
            }
            None => {
                if bucket.len() < K_ENTRIES_PER_BUCKET {
                    bucket.push(node_info);
                } else {
                    // go through bucket, pinging nodes, replace one
                    // that doesn't respond.
                }
            }
        }
    }

    /// Lookup the nodes closest to item in this table
    ///
    /// NOTE: This method is a really stupid, linear time search. I can't find
    /// info on how to use the buckets effectively to solve this.
    pub fn closest_nodes(&self, item: Key, count: usize) -> Vec<KnownNode> {
        if count == 0 {
            return Vec::new();
        }
        let mut ret = Vec::with_capacity(count);
        for bucket in &self.buckets {
            for node_info in bucket {
                ret.push(KnownNode {
                    node: node_info.clone(),
                    distance: node_info.id.dist(item),
                });
            }
        }
        ret.sort_by(|a, b| a.distance.cmp(&b.distance));
        ret.truncate(count);
        ret
    }

    pub fn remove(&mut self, node_info: &Node) {
        let bucket_index = self.lookup_bucket_index(node_info.id);
        if let Some(item_index) = self.buckets[bucket_index]
            .iter()
            .position(|x| x == node_info)
        {
            self.buckets[bucket_index].remove(item_index);
        } else {
            warn!("Tried to remove routing entry that doesn't exist.");
        }
    }

    fn lookup_bucket_index(&self, item: Key) -> usize {
        self.node_info.id.dist(item).zeroes_in_prefix()
    }

    pub fn print(&self) {
        info!("{:?}", self.buckets);
    }
}
