use redis_starter_rust::{MemoryDb, Server};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let db = MemoryDb::default();
    let addr = "127.0.0.1:6379";
    let server = Server::new(addr, &db);

    server.run().await;
}
