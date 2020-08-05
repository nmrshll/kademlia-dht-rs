use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::key::Key;
use crate::rout2::Node;
use crate::routing::{KnownNode, RoutingTable, K_ENTRIES_PER_BUCKET};

#[derive(Clone)]
pub struct Kad2<'a> {
    routes: Arc<Mutex<RoutingTable>>, // replace with interior mut Sync handle
    store: Arc<Mutex<HashMap<String, String>>>,
    node_self: Node<'a>,
}
impl<'a> Kad2<'a> {
    pub async fn new(node_self: &'a Node<'a>) -> Result<Self, Box<dyn Error>> {
        let addr: SocketAddr = ([0, 0, 0, 0], 8908).into(); // TODO config::port()
        let mut listener = TcpListener::bind(&addr).await?;

        let node_self = Node::new_self(&addr)?;

        // let node_info = Node {
        //     id: Key::random(),
        //     addr: socket.local_addr().unwrap().to_string(), // err: failed to retrieve local addr
        //     net_id: net_id,
        // };

        Ok(Kad2 {
            routes: Arc::new(Mutex::new(RoutingTable::new(node_self.clone()))),
            store: Arc::new(Mutex::new(Hashmap < String, String > ::new())),
            node_self,
        })
    }
    pub fn start(&self, bootstrap: Option<Node>) {}
}
