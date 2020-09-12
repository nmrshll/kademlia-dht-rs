// use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum Command {
    Get { key: String },
    Set { key: String, val: String },
}

pub struct State {
    map: HashMap<String, String>,
}
impl State {
    pub fn new() -> Self {
        State {
            map: HashMap::new(),
        }
    }
    pub fn start(mut self) -> StateHandle {
        // A channel to send commands to the state manager task
        let (tx, mut rx) = mpsc::channel(32);
        let task_state = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                use crate::state2::Command::*;
                match cmd {
                    Get { key } => {
                        self.map.get(&key);
                    }
                    Set { key, val } => {
                        self.map.insert(key, val);
                    }
                }
            }
        });
        return StateHandle {
            task: task_state,
            cmd_chan: tx,
        };
    }
}

pub struct StateHandle {
    task: JoinHandle<()>,
    cmd_chan: mpsc::Sender<Command>,
}
impl StateHandle {
    pub fn split(self) -> (JoinHandle<()>, mpsc::Sender<Command>) {
        return (self.task, self.cmd_chan);
    }
}
