use actix_web::{web, Scope};

pub mod auth;
mod notes;
mod users;
mod groups;

pub fn get_api() -> Scope {
    web::scope("/api")
        .service(
            web::scope("/notes")
                .service(
                    web::resource("/")
                        .route(web::get().to_async(notes::get_user_notes))
                        .route(web::post().to_async(notes::insert)))
                .service(
                    web::resource("/public")
                        .route(web::get().to_async(notes::get_public))))
                .service(
                    web::resource("/{id}")
                        .route(web::patch().to_async(notes::update_note)))
        .service(
            web::scope("/groups")
                .service(
                    web::resource("/")
                        .route(web::get().to_async(groups::get_user_groups))
                        .route(web::post().to_async(groups::insert)))
                .service(
                    web::resource("/join")
                        .route(web::post().to_async(groups::join))
                        .route(web::delete().to_async(groups::leave)))
        )
        .service(
            web::scope("/auth")
                .service(
                    web::resource("/")
                        .route(web::post().to_async(auth::login))
                        .route(web::delete().to_async(auth::logout))
                        .route(web::get().to_async(auth::get_me)),
                )
                .service(
                    web::resource("/register/")
                        .route(web::post().to_async(auth::register)))
                .service(
                    web::resource("/register/{uuid}")
                        .route(web::post().to_async(auth::confirm_registration))))
}
