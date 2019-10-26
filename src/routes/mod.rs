use actix_web::{web, Scope};

mod notes;
// mod auth;
// mod users;


pub fn get_api() -> Scope {
    web::scope("/api")
        .service(
            web::scope("/notes")
                .service(
                    web::resource("/")
                        .route(web::get().to_async(notes::get_list))
                        .route(web::post().to_async(notes::insert))
                ),
                
        )
}