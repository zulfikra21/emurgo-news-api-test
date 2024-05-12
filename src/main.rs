mod config;
mod controllers;
mod routes;
mod models;
mod grapqhl;

use actix::Actor;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use config::socket_server;
use crate::controllers::news;
use actix_cors::Cors;
use actix_files as fs;
use actix_web::{http, web, cookie::Key, App, HttpServer};
use controllers::home;
use diesel::{r2d2::{ConnectionManager, Pool}, PgConnection};
use dotenv::dotenv;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = std::env::var("PORT")
        .expect("Port is not defined").parse::<u16>().unwrap();
    let server = socket_server::ChatServer::default().start();
    println!("Server started on port {}", port);
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "DELETE","PUT"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .service(fs::Files::new("/public", "public").show_files_listing())
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .service(
                web::resource("/graphql")
                    .route(web::post().to(grapqhl::root::handler))   
            )
            // .app_data(web::Data::new(establish_connection()))
            .app_data(web::Data::new(server.clone()))
            .service(home::index)
            // news API
            .service(news::index)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
