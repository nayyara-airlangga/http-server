use std::{collections::HashMap, error::Error, str::FromStr};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

use crate::message::HttpMethod;

pub struct HttpServer {
    pub routes: HashMap<&'static str, HttpMethod>,
    pub listener: Option<TcpListener>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            listener: None,
        }
    }

    pub async fn bind(
        mut self,
        host: impl AsRef<str>,
        port: impl AsRef<str>,
    ) -> Result<Self, Box<dyn Error>> {
        let dst = format!("{}:{}", host.as_ref(), port.as_ref());

        self.listener = Some(TcpListener::bind(&dst).await?);
        println!("Server listening on {dst}");

        Ok(self)
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        if let Some(listener) = &self.listener {
            loop {
                let (stream, address) = listener.accept().await?;
                println!("Connection established from {address}");

                let mut reader = BufReader::new(stream);

                tokio::spawn(async move {
                    let mut start = String::new();
                    if let Err(_) = reader.read_line(&mut start).await {
                        return Err("Failed to read startline");
                    }

                    let start_parts = start.split_whitespace().collect::<Vec<&str>>();

                    if start_parts.len() != 3 {
                        return Err("Invalid HTTP request".into());
                    }

                    let (method, _path) = (start_parts[0], start_parts[1]);

                    if let Err(msg) = HttpMethod::from_str(method) {
                        let body = format!(
                            "HTTP/1.1 405 Method Not Allowed\r\nContent-Type: text/plain\r\n\r\n{msg}"
                        );
                        println!("{body}");
                        if let Err(_) = reader.write_all(body.as_bytes()).await {
                            return Err("Failed to write response".into());
                        }
                    }

                    Ok(())
                });
            }
        } else {
            return Err("Server has not made a valid connection".into());
        }
    }
}
