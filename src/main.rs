#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http::header, middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::env;

use crate::routes::auth;
use routes::get_api;

mod errors;
mod models;
mod routes;
mod schema;

fn main() {
    dotenv::dotenv().ok();
    if env::var("RUST_LOG").ok().is_none() {
        std::env::set_var("RUST_LOG", "conduit=debug,actix_web=info");
    }

    let sys = actix::System::new("conduit");

    let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS is not set");
    let fronend_address = env::var("FRONTEND_ADDRESS").expect("FRONTEND_ADDRESS is not set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(web::PayloadConfig::new(1 << 25))
            .data(web::JsonConfig::default().limit(1024 * 1024 * 50))
            .wrap(
                Cors::new()
                    .allowed_origin(&fronend_address)
                    .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "PATCH"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(auth::SECRET_KEY.as_bytes())
                    .name("auth")
                    .path("/")
                    .max_age_time(chrono::Duration::days(1))
                    .secure(false), // https
            ))
            .service(get_api())
    })
    .bind(&bind_address)
    .unwrap_or_else(|_| panic!("Could not bind address {}", &bind_address))
    .start();
    println!("Server started at {}", &bind_address);
    let _ = sys.run();
}
