use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

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
                Ok((mut stream, _)) => {
                    println!("Accepted connection from {:?}", stream.peer_addr().unwrap());

                    tokio::spawn(async move {
                        let mut buf = [0; 512];
                        loop {
                            let n = stream.read(&mut buf).await.unwrap();

                            if n == 0 {
                                break;
                            }

                            let actions = Action::from_bytes(&buf[..n]);
                            println!("Received actions: {:?}", actions);

                            for action in actions {
                                match action {
                                    Action::Ping => {
                                        stream.write(b"+PONG\r\n").await.unwrap();
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    println!("Failed to accept connection: {:?}", e);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Ping,
}

impl Action {
    pub fn from_str(action: &str) -> Vec<Self> {
        let lines = action.lines();
        let mut actions = Vec::new();
        for line in lines {
            match line {
                "PING" => actions.push(Self::Ping),
                _ => {}
            }
        }
        actions
    }

    pub fn from_bytes(action: &[u8]) -> Vec<Self> {
        let action = String::from_utf8(action.to_vec()).unwrap();
        Self::from_str(&action)
    }
}

#[cfg(test)]
mod tests {
    use crate::Action;

    #[test]
    fn parse_actions_from_str() {
        let req = "PING\nPING\n";
        let actions = Action::from_str(req);
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn parse_action_from_str_ping() {
        let req = "PING";
        let actions = Action::from_str(req);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Ping);
    }

    #[test]
    fn parse_action_from_str_empty() {
        let req = "";
        let actions = Action::from_str(req);
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn parse_action_from_bytes_ping() {
        let req = b"PING";
        let actions = Action::from_bytes(req);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Ping);
    }

    #[test]
    fn parse_action_from_bytes_empty() {
        let req = b"";
        let actions = Action::from_bytes(req);
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn parse_action_from_bytes_ping_newline() {
        let req = b"PING\n";
        let actions = Action::from_bytes(req);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::Ping);
    }

    #[test]
    fn parse_action_from_bytes_ping_newline_ping() {
        let req = b"PING\nPING";
        let actions = Action::from_bytes(req);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], Action::Ping);
        assert_eq!(actions[1], Action::Ping);
    }
}
