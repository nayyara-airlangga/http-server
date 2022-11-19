use http_server::server::HttpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = HttpServer::new().bind("127.0.0.1", "8000").await?;

    server.run().await
}
