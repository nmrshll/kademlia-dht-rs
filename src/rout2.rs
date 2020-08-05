use std::cmp::Ordering;
use std::error::Error;
use std::net::SocketAddr;

use super::key::{Distance, Key, KEY_LEN};

pub const N_BUCKETS: usize = KEY_LEN * 8;
pub const K_ENTRIES_PER_BUCKET: usize = 8;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node<'a> {
    pub id: Key,
    pub addr: SocketAddr, // TODO &'a (prob with Deserialize)
    pub net_id: String,   // TODO move to RoutingTable ?
}
impl<'a> Node<'a> {
    pub fn new_self(addr: &'a SocketAddr) -> Result<Self, Box<dyn Error>> {
        Ok(Node {
            id: Key::random(),
            addr: addr.clone(),
            net_id: String::from("tender_test_net"),
        })
    }
}

/// KnownNode exists to store metadata about known other nodes on the network
/// At the moment, we score (and sort) other nodes only by distance
/// Longer term, we want to include other factors into the score (uptime, public IP?, trust score, well known nodes)
/// There's no consensus on score. Each peer maintains is own score list independently.
#[derive(Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct KnownNode<'a> {
    pub node: Node<'a>, // TODO &'a (prob with Deserialize)
    pub distance: Distance,
}
impl<'a> PartialEq for KnownNode<'a> {
    fn eq(&self, other: &KnownNode) -> bool {
        self.distance.eq(&other.distance)
    }
}
impl<'a> PartialOrd for KnownNode<'a> {
    fn partial_cmp(&self, other: &KnownNode) -> Option<Ordering> {
        Some(other.distance.cmp(&self.distance))
    }
}
impl<'a> Ord for KnownNode<'a> {
    fn cmp(&self, other: &KnownNode) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

/// RoutingTable keeps nodes sorted, gets updated on requests
#[derive(Debug)]
pub struct RoutingTable<'a> {
    node_self: &'a Node<'a>,
    buckets: Vec<Vec<Node<'a>>>, // TODO mutex with slot-level locking (i.e. inside the Vec<Vec<>>) (rather than on the whole routing table)
}
impl<'a> RoutingTable<'a> {
    pub fn new(node_self: &'a Node<'a>) -> Self {
        let mut buckets = Vec::new();
        for _ in 0..N_BUCKETS {
            buckets.push(Vec::new());
        }
        let mut ret = RoutingTable {
            node_self: &node_self,
            buckets: buckets,
        };
        ret.update(node_self.clone()); // TODO &'a
        ret
    }

    // pub fn distance_to(other_node: Node) -> Distance {}

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

    /// Lookup the nodes closest to target in this table
    ///
    /// NOTE: This method is a really basic, linear time search.
    /// TODO use buckets and distances instead
    pub fn closest_nodes(&self, target: Key, count: usize) -> Vec<KnownNode> {
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
            tracing::warn!("Tried to remove routing entry that doesn't exist.");
            // TODO return Error
        }
    }

    /// TODO document
    fn lookup_bucket_index(&self, item: Key) -> usize {
        self.node_self.id.dist(item).zeroes_in_prefix()
    }

    // TODO impl Debug
    pub fn print(&self) {
        tracing::info!("{:?}", self.buckets);
    }
}
