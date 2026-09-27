use std::io::{BufRead, BufReader, Error, Write};
use std::net::TcpListener;
use std::sync::mpsc;

pub struct Server {
    port: u16,
    content: String,
    accepted_keys: Option<Vec<String>>,
}

impl Server {
    pub fn new(port: u16, content: String, accepted_keys: Option<Vec<String>>) -> Self {
        Self {
            port,
            content,
            accepted_keys,
        }
    }

    fn update_content(&mut self, content: String) {
        self.content = content
    }

    fn parse_http_api_key_header(request: &Vec<String>) -> Option<&str> {
        let api_key_line = request.iter().find(|line| line.starts_with("X-API-KEY:"));
        if let Some(line) = api_key_line {
            let key = &line["X-API-KEY: ".len()..];
            Some(key)
        } else {
            None
        }
    }

    fn is_valid_request(&self, request: Vec<String>) -> bool {
        let api_keys = match self.accepted_keys {
            Some(ref keys) => keys,
            None => return true,
        };

        match Self::parse_http_api_key_header(&request) {
            Some(key) => api_keys.contains(&key.to_string()),
            None => false,
        }
    }

    pub fn serve(&mut self, reciever: mpsc::Receiver<String>) -> Result<(), Error> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;

        for stream in listener.incoming() {
            if let Ok(m) = reciever.try_recv() {
                self.update_content(m)
            }

            let mut stream = stream?;

            let buf_reader = BufReader::new(&stream);
            let request: Vec<_> = buf_reader
                .lines()
                .map(|result| result.ok().get_or_insert("".to_string()).clone())
                .take_while(|line| !line.is_empty())
                .collect();

            println!("[cred-server] New request!\nRequest info: {:#?}\n", request);

            let response = match self.is_valid_request(request) {
                true => {
                    let status_line = "HTTP/1.1 200 OK";
                    let length = self.content.len();
                    format!(
                        "{status_line}\r\nContent-Length:{length}\r\n\r\n{}\r\n",
                        self.content
                    )
                }
                false => {
                    let status_line = "HTTP/1.1 403 Forbidden";
                    let content = "403 Forbidden\nYou are not allowed to view this data.";
                    let length = content.len();
                    format!(
                        "{status_line}\r\nContent-Length:{length}\r\n\r\n{}\r\n",
                        content
                    )
                }
            };

            stream.write_all(response.as_bytes())?;

            println!("Responded with:\n{}", response);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn serves_content() {
        use std::process::Command;
        use std::sync::mpsc;
        use std::thread;

        let (_tx, rx) = mpsc::channel();

        let find_open_port_bind = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = find_open_port_bind.local_addr().unwrap().port();
        drop(find_open_port_bind);

        thread::spawn(move || {
            Server::new(port, "test".to_string(), None)
                .serve(rx)
                .unwrap();
        });

        // Make sure curl doesn't start before server starts
        thread::sleep(Duration::from_millis(1));

        let result = Command::new("curl")
            .arg(format!("127.0.0.1:{}", port))
            .output()
            .unwrap();

        assert_eq!(b"test", result.stdout.as_slice());
    }
}
