pub mod resp;
use anyhow::Result;
use resp::Value;
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn run(self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        loop {
            let stream = listener.accept().await;
            match stream {
                Ok((stream, _)) => {
                    println!("Accepted connection from {:?}", stream.peer_addr().unwrap());

                    tokio::spawn(async move {
                        handle_conn(stream).await;
                    });
                }
                Err(e) => {
                    println!("Failed to accept connection: {:?}", e);
                }
            }
        }
    }
}

// *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n

async fn handle_conn(stream: TcpStream) {
    let mut handler = resp::RespHandler::new(stream);

    println!("Starting read loop");

    loop {
        let value = match handler.read_value().await {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to read value: {:?}", e);
                break;
            }
        };
        println!("Got value {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();

            match command.as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };

        println!("Sending value {:?}", response);

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string")),
    }
}
