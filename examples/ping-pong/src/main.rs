mod client;
mod messages;
mod server;

use std::thread;
use std::time::Duration;

const ADDR: &str = "127.0.0.1:8080";

fn main() {
    // Start the server in a separate thread
    thread::spawn(|| {
        server::start_server(ADDR).expect("Server failed");
    });

    // Give the server a moment to start up
    thread::sleep(Duration::from_secs(1));

    // Start the client
    client::start_client(ADDR).expect("Client failed");
}
