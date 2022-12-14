use std::{error::Error, sync::Arc};

use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::Mutex,
};

use crate::{
    message::{HttpRequest, HttpResponse, HttpStatus},
    router::Router,
    service::Service,
};

pub struct HttpServer {
    pub router: Arc<Mutex<Router>>,
    pub listener: Option<TcpListener>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            router: Arc::new(Mutex::new(Router::new())),
            listener: None,
        }
    }

    pub fn serve(mut self, router: Router) -> Self {
        self.router = Arc::new(Mutex::new(router));
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
                let router = Arc::clone(&self.router);

                tokio::spawn(async move {
                    let req = HttpRequest::from_stream_reader(Arc::clone(&reader)).await?;
                    let mut reader = reader.lock().await;

                    let router = router.lock().await;
                    let route = router.routes().get(req.path().as_str());
                    if let Some(route) = route {
                        if let Some(handler) = route.methods().get(req.method().as_ref()) {
                            let res = handler.call(req).await;

                            if let Err(_) = reader.write_all(res.to_string().as_bytes()).await {
                                return Err("Failed to write response");
                            }
                        } else {
                            let mut res = HttpResponse::new(HttpStatus::MethodNotAllowed);
                            res.set_header("Content-Type", "text/plain");
                            res.set_body("Method not allowed");

                            if let Err(_) = reader.write_all(res.to_string().as_bytes()).await {
                                return Err("Failed to write response");
                            }
                        }
                    } else {
                        let mut res = HttpResponse::new(HttpStatus::NotFound);
                        res.set_header("Content-Type", "text/plain");
                        res.set_body("Page not found");

                        if let Err(_) = reader.write_all(res.to_string().as_bytes()).await {
                            return Err("Failed to write response");
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
