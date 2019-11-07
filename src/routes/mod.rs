use actix_web::{web, Scope};

pub mod auth;
mod notes;
mod users;

pub fn get_api() -> Scope {
    web::scope("/api")
        .service(
            web::scope("/notes").service(
                web::resource("/")
                    .route(web::get().to_async(notes::get_list))
                    .route(web::post().to_async(notes::insert)),
            ),
        )
        .service(
            web::scope("/auth")
                .service(
                    web::resource("/")
                        .route(web::post().to_async(auth::login))
                        .route(web::delete().to_async(auth::logout))
                        .route(web::get().to_async(auth::get_me)),
                )
                .service(web::resource("/register/").route(web::post().to_async(auth::register)))
                .service(
                    web::resource("/register/{uuid}")
                        .route(web::post().to_async(auth::confirm_registration)),
                ),
        )
}
