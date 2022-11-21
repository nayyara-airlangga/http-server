use http_server::{
    message::{HttpRequest, HttpResponse, HttpStatus},
    router::{get, Router},
    server::HttpServer,
};

async fn get_handler(_req: HttpRequest) -> HttpResponse {
    let mut res = HttpResponse::new(HttpStatus::OK);
    res.set_header("Content-Type", "text/plain");
    res.set_body("LOLOLOL");
    res
}

async fn post_handler(req: HttpRequest) -> HttpResponse {
    let mut res = HttpResponse::new(HttpStatus::OK);
    res.set_header("Content-Type", "text/plain");
    res.set_body(req.body());
    res
}

async fn index(_req: HttpRequest) -> HttpResponse {
    let mut res = HttpResponse::new(HttpStatus::OK);
    res.set_header("Content-Type", "text/plain");
    res.set_body("OK!");
    res
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .route("/", get(index))
        .route("/mabar", get(get_handler).post(post_handler));

    let server = HttpServer::new()
        .serve(router)
        .bind("127.0.0.1", "8000")
        .await?;

    server.run().await
}
