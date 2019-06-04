#[macro_use]
extern crate serde_derive;

mod kademlia;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
pub mod test {
    use super::*;
    use kademlia;

    #[test]
    fn test_debug_key() {
        let key = kademlia::Key::random();
        dbg!(key);
    }
}
