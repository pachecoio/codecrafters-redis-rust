use redis_starter_rust::Server;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let server = Server::new("127.0.0.1:6379");

    server.run();
}
