use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;
use r2d2::Pool;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{User, PublicUser};

type SqlPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn get_user(
    uuid: web::Path<Uuid>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::users::dsl::*;
    web::block(move || -> Result<PublicUser, ServiceError> {
        let conn = pool.get().unwrap();
        let uuid = uuid.into_inner().to_string();
        let user = users
            .filter(id.eq(&uuid))
            .first::<User>(&conn)?;
        Ok(PublicUser::from(user))
    })
    .then(|res| match res {
        Ok(t) => Ok(HttpResponse::Ok().json(t)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}