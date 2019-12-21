use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;
use r2d2::Pool;

use crate::errors::ServiceError;
use crate::models::{LoggedUser, Group, NewGroup, GroupLink};

type SqlPool = Pool<ConnectionManager<SqliteConnection>>;



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
    use crate::schema::group_links::dsl::id as l_id;
    web::block(move || -> Result<Group, ServiceError> {
        let conn = pool.get().unwrap();
        let target = target.into_inner();
        let group = groups.filter(g_id.eq(&target.id)).first::<Group>(&conn)?;
        let link = group_links
            .filter(group_id.eq(&target.id).and(user_id.eq(&user.id)))
            .first::<GroupLink>(&conn)?;
        diesel::delete(group_links.filter(l_id.eq(&link.id))).execute(&conn)?;
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