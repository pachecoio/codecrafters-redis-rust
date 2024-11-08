use std::{
    io::{Read, Write},
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

    pub fn run(self) {
        let listener = TcpListener::bind(&self.addr).unwrap();

        for stream in listener.incoming() {
            println!("connection established!");
            let mut stream = stream.unwrap();
            let mut buffer = [0; 1024];

            // keep reading the stream
            loop {
                let bytes_read = stream.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break;
                }

                let actions = Action::from_bytes(&buffer[..bytes_read]);
                for action in actions {
                    match action {
                        Action::Ping => {
                            stream.write(b"+PONG\r\n").unwrap();
                        }
                    }
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
