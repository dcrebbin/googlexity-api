use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;

mod constants;
mod middleware;
mod models;
mod routes;
mod services;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string()) // Default to 8080 if PORT is not set
        .parse::<u16>()
        .expect("PORT must be a valid number");
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::guard_middleware::ApiKeyMiddleware)
            .wrap(Logger::new("%a %{User-Agent}i %r %s %b %T")) // Single, more detailed logger
            .service(web::scope("/api").configure(|r| {
                r.route("/test", web::post().to(routes::test::test_route));
                r.route(
                    "/google-cloud-auth",
                    web::get().to(routes::test::test_google_auth_route),
                );
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
