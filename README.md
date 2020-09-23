# kademlia-dht-rs

Kademlia DHT implementation in Rust

#### What is it ? What can I build with it ?

A DHT (Distributed hash table) is a building block for peer-to-peer networks. It provides both a distributed key-value store, where entries are stored spread across peers, and connectivity/networking between peers.
Many notable p2p networks use one: for instance, Bittorrent uses one for the key-value store, to store and retrieve files, Ethereum and IPFS use one to create an overlay network (a network abstraction layer, built over UDP/IP).

If you're writing a p2p network node (client and server), this crate helps you add a DHT to it, connect peers/nodes together, and provides distributed storage.

#### What about alternative implementations ?

- leejunseok
- dtantsur

## Usage

Requires: `Rust 1.34+`

Install by adding to your `Cargo.toml`'s `[dependencies]`: `kademlia = { git = "https://github.com/nmrshll/kademlia-dht-rs" }`

## Testing

`cargo test`
