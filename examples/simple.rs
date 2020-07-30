use kademlia::{Kademlia, Key, Node};

fn main() {
    println!("Hello, world!");

    let kad = Kademlia::start(String::from("test_net"), Key::random(), "127.0.0.1:0", None);
    // let mut dummy_info = Node {
    //     net_id: String::from("test_net"),
    //     addr: String::from("asdfasdf"),
    //     id: Key::random(),
    // };

    let input = std::io::stdin();
    loop {
        let mut buffer = String::new();
        if input.read_line(&mut buffer).is_err() {
            break;
        }
        // let args = buffer.trim_right().split(' ').collect::<Vec<_>>();
        println!("Press Enter to ping self");

        // auto-ping
        let mut dummy_info = Node {
            net_id: String::from("test_net"),
            addr: String::from("asdfasdf"),
            id: Key::random(),
        };

        let pingRes = kad.ping(dummy_info.clone());

        // match args[0].as_ref() {
        //     "p" => {
        //         dummy_info.addr = String::from(args[1]);
        //         dummy_info.id = Key::from(String::from(args[2]));
        //         println!("{:?}", handle.ping(dummy_info.clone()));
        //     }
        //     "s" => {
        //         dummy_info.addr = String::from(args[1]);
        //         dummy_info.id = Key::from(String::from(args[2]));
        //         println!(
        //             "{:?}",
        //             handle.store(
        //                 dummy_info.clone(),
        //                 String::from(args[3]),
        //                 String::from(args[4])
        //             )
        //         );
        //     }
        //     "fn" => {
        //         dummy_info.addr = String::from(args[1]);
        //         dummy_info.id = Key::from(String::from(args[2]));
        //         println!(
        //             "{:?}",
        //             handle.find_node(dummy_info.clone(), Key::from(String::from(args[3])))
        //         );
        //     }
        //     "fv" => {
        //         dummy_info.addr = String::from(args[1]);
        //         dummy_info.id = Key::from(String::from(args[2]));
        //         println!(
        //             "{:?}",
        //             handle.find_value(dummy_info.clone(), String::from(args[3]))
        //         );
        //     }
        //     "ln" => {
        //         println!(
        //             "{:?}",
        //             handle.lookup_nodes(Key::from(String::from(args[1])))
        //         );
        //     }
        //     "lv" => {
        //         println!("{:?}", handle.lookup_value(String::from(args[1])));
        //     }
        //     "put" => {
        //         println!(
        //             "{:?}",
        //             handle.put(String::from(args[1]), String::from(args[2]))
        //         );
        //     }
        //     "get" => {
        //         println!("{:?}", handle.get(String::from(args[1])));
        //     }
        //     _ => {
        //         println!("no match");
        //     }
        // }
    }
}
