use actix_web::{web, HttpResponse, error::BlockingError};
use r2d2::Pool;
use futures::Future;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use crate::models::{ Note, NewNote, LoggedUser};
use crate::errors::ServiceError;

type SqlPool = Pool<ConnectionManager<SqliteConnection>>;


pub fn get_list(
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    web::block(
        move || -> Result<Vec<Note>, ServiceError> {
            let conn = pool.get().unwrap();
            let list_of_notes = notes.filter(user_id.eq(user.id)).load::<Note>(&conn)?;
            Ok(list_of_notes)
        }
    )
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    )
}


pub fn insert(
    user: LoggedUser,
    note: web::Json<NewNote>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    web::block(
        move || -> Result<Note, ServiceError> {
            let conn = pool.get().unwrap();
            let note = Note::from(note.into_inner(), user);
            diesel::insert_into(notes).values(&note).execute(&conn)?;
            Ok(note)
        }
    )
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    )
}