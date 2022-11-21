use std::{collections::HashMap, error::Error, sync::Arc};

use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::Mutex,
};

use crate::{
    message::{HttpMethod, HttpRequest, HttpResponse, HttpStatus},
    router::Router,
};

pub struct HttpServer {
    pub router: Router,
    pub routes: HashMap<&'static str, HttpMethod>,
    pub listener: Option<TcpListener>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            routes: HashMap::new(),
            listener: None,
        }
    }

    pub fn serve(mut self, router: Router) -> Self {
        self.router = router;
        self
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

                let reader = Arc::new(Mutex::new(BufReader::new(stream)));

                tokio::spawn(async move {
                    let req = HttpRequest::from_stream_reader(Arc::clone(&reader)).await?;

                    let mut reader = reader.lock().await;
                    match req.method() {
                        HttpMethod::Get => {
                            let mut res = HttpResponse::new(HttpStatus::OK);
                            res.set_header("Content-Type", "text/html");
                            res.set_body(format!("<p>Route {} is accessed</p>", req.path()));

                            if let Err(_) = reader.write_all(res.to_string().as_bytes()).await {
                                return Err("Failed to write response");
                            }
                        }
                        HttpMethod::Post => {
                            let mut res = HttpResponse::new(HttpStatus::OK);
                            res.set_header("Content-Type", "text/plain");

                            let encoded = base64::encode(req.body());
                            res.set_body(encoded);

                            if let Err(_) = reader.write_all(res.to_string().as_bytes()).await {
                                return Err("Failed to write response");
                            }
                        }
                    };

                    Ok(())
                });
            }
        } else {
            return Err("Server has not made a valid connection".into());
        }
    }
}
