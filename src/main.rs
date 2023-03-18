use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_client(stream);
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();

    for _ in reader.lines() {
        stream.write(b"+PONG\r\n").unwrap();
    }
}

