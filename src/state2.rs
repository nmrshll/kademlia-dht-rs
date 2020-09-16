// use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
//
use crate::rout2::{Node, RoutingTable};
use crate::Key;

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<Result<T, StateErr>>;
/// A command for the Key-value store task
#[derive(Debug)]
pub enum KvCmd {
    Get {
        key: String,
        resp: Responder<Option<String>>,
    },
    Set {
        key: String,
        val: String,
        resp: Responder<()>,
    },
}
/// A command for the router task
#[derive(Debug)]
pub enum RouterCmd {
    GetClosestNodes { resp: Responder<()> },
    Update,
    Remove,
    // TODO maybe more ?
}

pub struct State {
    // kv: HashMap<String, String>,
// router: RoutingTable,
}
impl State {
    // pub fn new(node_self: Node) -> Self {
    //     State {
    //         kv: HashMap::new(),
    //         router: RoutingTable::new(&node_self),
    //     }
    // }
    pub fn start(node_self: Node) -> (StateTasks, CmdChans) {
        // Create new state
        let mut kv: HashMap<String, String> = HashMap::new();
        let router = RoutingTable::new(&node_self);

        // A channel to send commands to the kv manager task
        let (tx_kv, mut rx) = mpsc::channel(32);
        let task_kv = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    KvCmd::Get { key, resp } => {
                        let res = kv.get(&key).cloned();
                        // Ignore errors
                        let _ = resp.send(Ok(res)); // TODO error handling
                    }
                    KvCmd::Set { key, val, resp } => {
                        let res = kv.insert(key, val);
                        // Ignore errors
                        let _ = resp.send(Ok(())); // TODO error handling
                    }
                }
            }
        });

        // A channel to send commands to the router manager task
        let (tx_router, mut rx) = mpsc::channel(32);
        let task_router = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    RouterCmd::GetClosestNodes { resp } => {
                        let res = router.closest_nodes(Key::random(), 3); // TODO not random

                        // Ignore errors
                        let _ = resp.send(Ok(())); // TODO error handling
                    }
                    _ => unimplemented!(), // TODO implement other varidants
                }
            }
        });

        return (
            StateTasks {
                kv: task_kv,
                router: task_router,
            },
            (tx_kv, tx_router),
        );
    }
}
pub struct StateTasks {
    pub kv: JoinHandle<()>,
    pub router: JoinHandle<()>,
}
// pub struct CmdChans {
//     kv: mpsc::Sender<KvCmd>,
// }

// TODO re-group into structs since rust does auto-splitting into distinct tasks
// after all no it doesn't. need to clone before spawn but that's fine
pub type KvTask = JoinHandle<()>;
pub type KvCmdChan = mpsc::Sender<KvCmd>;
pub type RouterTask = JoinHandle<()>;
pub type RouterCmdChan = mpsc::Sender<RouterCmd>;

// pub type StateTasks = (KvTask, RouterTask);
pub type CmdChans = (KvCmdChan, RouterCmdChan);
// pub type StateTask = JoinHandle<()>;
// pub type KvCmdChan = mpsc::Sender<KvCmd>;

// pub struct StateHandle {
//     task: JoinHandle<()>,
//     cmd_chan: mpsc::Sender<Command>,
// }
// impl StateHandle {
//     pub fn split(self) -> (JoinHandle<()>, mpsc::Sender<Command>) {
//         return (self.task, self.cmd_chan);
//     }
// }

use thiserror::Error;
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StateErr {
    #[error("unknown StateManager error")]
    Unknown,
}