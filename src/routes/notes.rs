use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;
use r2d2::Pool;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{LoggedUser, NewNote, Note, NotePatch, GroupLink};

type SqlPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn get_user_notes(
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    web::block(move || -> Result<Vec<Note>, ServiceError> {
        let conn = pool.get().unwrap();
        let list_of_notes = notes.filter(user_id.eq(user.id)).load::<Note>(&conn)?;
        Ok(list_of_notes)
    })
    .then(|res| match res {
        Ok(t) => Ok(HttpResponse::Ok().json(t)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn get_public(
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    web::block(move || -> Result<Vec<Note>, ServiceError> {
        let conn = pool.get().unwrap();
        let list_of_notes = notes.filter(public.eq(1)).load::<Note>(&conn)?;
        Ok(list_of_notes)
    })
    .then(|res| match res {
        Ok(t) => Ok(HttpResponse::Ok().json(t)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn get_note (
    user: LoggedUser,
    uuid: web::Path<Uuid>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    use crate::schema::group_links::group_id as l_g_id;
    web::block(move || -> Result<Note, ServiceError> {
        let conn = pool.get().unwrap();
        let uuid = uuid.into_inner().to_string();
        let note = notes
            .filter(id.eq(&uuid))          
            .first::<Note>(&conn)?;
        if note.user_id != user.id && note.public != 1 {
            //check groups
            match &note.group_id {
                Some(gid) => {
                    let mut link = GroupLink::belonging_to(&user)
                        .filter(l_g_id.eq(gid))
                        .load::<GroupLink>(&conn)?;
                    if let Some(_) = link.pop() {
                        return Ok(note);
                    } else {
                        return Err(ServiceError::Forbidden);
                    }
                },
                None => return Err(ServiceError::Forbidden),
            }   
        }
        Ok(note)
    })
    .then(|res| match res {
        Ok(t) => Ok(HttpResponse::Ok().json(t)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn update_note(
    user: LoggedUser,
    uuid: web::Path<Uuid>,
    note: web::Json<NotePatch>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    web::block(move || -> Result<Note, ServiceError> {
        let conn = pool.get().unwrap();
        let uuid = uuid.into_inner().to_string();
        let note = note.into_inner();
        let mut result = notes
            .filter(id.eq(&uuid))
            .load::<Note>(&conn)?;
        if let Some(db_note) = result.pop(){
            if &db_note.user_id != &user.id {
                return Err(ServiceError::Forbidden);
            }
            diesel::update(&db_note)
                .set(&note)
                .execute(&conn)?;
            let updated_note = notes
                .filter(id.eq(&uuid))
                .first::<Note>(&conn)?;
            return Ok(updated_note);
        }
        Err(ServiceError::BadRequest(
            String::from("Invalid note identifiers!")
        ))
    })
    .then(|res| match res {
        Ok(t) => Ok(HttpResponse::Ok().json(t)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}


pub fn insert(
    user: LoggedUser,
    note: web::Json<NewNote>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::notes::dsl::*;
    web::block(move || -> Result<Note, ServiceError> {
        let conn = pool.get().unwrap();
        let note = Note::from(note.into_inner(), user);
        diesel::insert_into(notes).values(&note).execute(&conn)?;
        Ok(note)
    })
    .then(|res| match res {
        Ok(t) => Ok(HttpResponse::Ok().json(t)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}
