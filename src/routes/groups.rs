use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;
use r2d2::Pool;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{LoggedUser, Group, NewGroup, GroupLink, Note, GroupedNotes};

type SqlPool = Pool<ConnectionManager<SqliteConnection>>;


pub fn group_notes(
    _: LoggedUser,
    uuid: web::Path<Uuid>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::groups::dsl::id as g_id;
    use crate::schema::notes::dsl::*;
    use crate::schema::notes::dsl::group_id as n_g_id;
    web::block(move || -> Result<GroupedNotes, ServiceError> {
        let conn = pool.get().unwrap();
        let uuid = uuid.into_inner().to_string();
        let group = groups.filter(g_id.eq(&uuid)).first::<Group>(&conn)?;
        let note_list = notes.filter(n_g_id.eq(&uuid)).load::<Note>(&conn)?;
        Ok(GroupedNotes {
            group, 
            notes: note_list
        })
    }) 
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}

pub fn users_groups_notes(
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::groups::dsl::id as g_id;
    use crate::schema::group_links::dsl::group_id as l_g_id;
    use crate::schema::notes::dsl::*;
    use crate::schema::notes::dsl::group_id as n_g_id;
    web::block(move || -> Result<Vec<GroupedNotes>, ServiceError> {
        let conn = pool.get().unwrap();
        let mut out: Vec<GroupedNotes> = vec![];
        let group_ids = GroupLink::belonging_to(&user)
            .select(l_g_id).load::<String>(&conn)?;
        let group_list = groups.filter(g_id.eq_any(&group_ids)).load::<Group>(&conn)?;
        let note_list = notes.filter(n_g_id.eq_any(&group_ids)).load::<Note>(&conn)?;
        let grouped_notes: Vec<Vec<Note>> = note_list.grouped_by(&group_list);
        let zipped: Vec<(Group, Vec<Note>)> = group_list
            .into_iter()
            .zip(grouped_notes)
            .collect();
        for grp in zipped {
            out.push(GroupedNotes {
                group: grp.0,
                notes: grp.1,
            });
        } 
        Ok(out)
        
    }) 
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}


pub fn get_user_groups(
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::groups::dsl::id as g_id;
    use crate::schema::group_links::dsl::*;
    web::block(move || -> Result<Vec<Group>, ServiceError> {
        let conn = pool.get().unwrap();
        let group_ids = GroupLink::belonging_to(&user)
            .select(group_id).load::<String>(&conn)?;
        let group_list = groups.filter(g_id.eq_any(&group_ids)).load::<Group>(&conn)?;
        Ok(group_list)
    })
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}


pub fn insert(
    new_group: web::Json<NewGroup>,
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    web::block(move || -> Result<Group, ServiceError> {
        let conn = pool.get().unwrap();
        let group = Group::from(new_group.into_inner(), user);
        diesel::insert_into(groups).values(&group).execute(&conn)?;
        Ok(group)
    })
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}

#[derive(Deserialize)]
pub struct GroupTarget {
    id: String,
}

pub fn join(
    target: web::Json<GroupTarget>,
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::groups::dsl::id as g_id;
    use crate::schema::group_links::dsl::*;
    web::block(move || -> Result<Group, ServiceError> {
        let conn = pool.get().unwrap();
        let target = target.into_inner();
        let group = groups.filter(g_id.eq(&target.id)).first::<Group>(&conn)?;
        let group_link = GroupLink::from(&group, &user);
        diesel::insert_into(group_links)
            .values(&group_link)
            .execute(&conn)?;
        Ok(group)
    })
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}

pub fn leave(
    target: web::Json<GroupTarget>,
    user: LoggedUser,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::groups::dsl::id as g_id;
    use crate::schema::group_links::dsl::*;
    web::block(move || -> Result<Group, ServiceError> {
        let conn = pool.get().unwrap();
        let target = target.into_inner();
        let group = groups.filter(g_id.eq(&target.id)).first::<Group>(&conn)?;
        diesel::delete(
            group_links.filter(group_id.eq(&target.id).and(user_id.eq(&user.id))))
            .execute(&conn)?;
        Ok(group)
    })
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}

pub fn delete(
    user: LoggedUser,
    uuid: web::Path<Uuid>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::groups::dsl::*;
    use crate::schema::groups::dsl::id as g_id;
    use crate::schema::group_links::dsl::*;
    web::block(move || -> Result<Group, ServiceError> {
        let conn = pool.get().unwrap();
        let target = uuid.into_inner().to_string();
        let group = groups.filter(g_id.eq(&target)).first::<Group>(&conn)?;
        if &group.created_by != &user.id {
            return Err(ServiceError::Forbidden);
        }
        diesel::delete(
            group_links.filter(group_id.eq(&target)))
            .execute(&conn)?;
        diesel::delete(
            groups.filter(g_id.eq(&target)))
            .execute(&conn)?;
        Ok(group)
    })
    .then(
        |res| match res {
            Ok(t) => Ok(HttpResponse::Ok().json(t)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            }
        }
    )
}