use http_server::{
    message::{HttpRequest, HttpResponse, HttpStatus},
    router::{Route, Router},
    server::HttpServer,
};

async fn lol(_req: HttpRequest) -> HttpResponse {
    let mut res = HttpResponse::new(HttpStatus::OK);
    res.set_header("Content-Type", "text/plain");
    res.set_body("LOLOLOL");
    res
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new().route("/mabar", Route::new().get(lol).get(lol));

    let server = HttpServer::new()
        .serve(router)
        .bind("127.0.0.1", "8000")
        .await?;

    server.run().await
}
