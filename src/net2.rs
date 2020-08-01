use anyhow::Error as AnyErr;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
// use std::collections::HashMap;
// use std::str;
// use std::sync::mpsc;
// use std::sync::mpsc::{Receiver, Sender};
// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::time::Duration;

// use super::kademlia::{Reply, Request};
// use super::key::Key;
// use super::routing::Node;
// use log::{debug, info, warn};

/// Max message length
pub const MESSAGE_LEN: usize = 8196;
/// Default timeout
pub const TIMEOUT: u64 = 5000;

/////

pub struct RpcClient {
    // stream: TcpStream,
}
impl RpcClient {
    pub fn new() -> Self {
        RpcClient {}
    }
    pub async fn send(&self) -> Result<(), AnyErr> {
        let rmsg = "yoyoyo";
        let enc_msg = serde_json::to_vec(rmsg).expect("failed serde");
        //TEMP
        let remote_addr: SocketAddr = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "52.20.16.20:30000".into())
            .parse()
            .expect("failed parsing remote addr");
        let mut stream = TcpStream::connect(remote_addr).await?;
        let (r, mut w) = stream.split();

        let result = w.write(b"hello world\n").await?;
        println!("wrote res: {}", result);

        // let resp = r.read().await;
        Ok(())
    }
}
