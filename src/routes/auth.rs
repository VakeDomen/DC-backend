use crate::email_service::send_mail;
use crate::errors::ServiceError;
use crate::models::{Invitation, LoggedUser, NewUser, User};

use actix_identity::Identity;
use actix_web::{
    dev::Payload, error::BlockingError, web, Error, FromRequest, HttpRequest, HttpResponse,
};
use argonautica::{Hasher, Verifier};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;
use r2d2::Pool;
use std::env;
use uuid::Uuid;
type SqlPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Result<LoggedUser, Error>;
    type Config = ();

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Some(identity) = Identity::from_request(req, pl)?.identity() {
            let user: LoggedUser = serde_json::from_str(&identity)?;
            return Ok(user);
        }
        Err(ServiceError::Unauthorized.into())
    }
}

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(8));
}

pub fn hash_password(password: &str) -> Result<String, ServiceError> {
    print!("{}", password);
    Hasher::default()
        .with_password(password)
        .with_secret_key(SECRET_KEY.as_str())
        .hash()
        .map_err(|err| {
            dbg!(err);
            ServiceError::InternalServerError
        })
}

pub fn verify(hash: &str, password: &str) -> Result<bool, ServiceError> {
    Verifier::default()
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(SECRET_KEY.as_str())
        .verify()
        .map_err(|err| {
            dbg!(err);
            ServiceError::Unauthorized
        })
}

pub fn register(
    new_user: web::Json<NewUser>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    use crate::schema::invitations::dsl::*;
    use crate::schema::users::dsl::*;
    web::block(move || -> Result<Option<Invitation>, ServiceError> {
        let frotend_target =
            env::var("REGISTRATION_CONFIRMATION_URL").expect("BIND_ADDRESS is not set");
        let frontend_address = env::var("FRONTEND_ADDRESS").expect("FRONTEND_ADDRESS is not set");
        let conn = pool.get().unwrap();
        let user = User::from(new_user.into_inner());
        let invitation = Invitation::from_user(&user);
        diesel::insert_into(users).values(&user).execute(&conn)?;
        diesel::insert_into(invitations)
            .values(&invitation)
            .execute(&conn)?;
        send_mail(
            user.email.clone(),
            String::from("Confirm registration for InnoReserve"),
            frontend_address + &frotend_target + &invitation.id,
        );
        Ok(Some(invitation))
    })
    .then(|res| match res {
        Ok(location) => match location {
            Some(t) => Ok(HttpResponse::Ok().json(t)),
            None => Ok(HttpResponse::InternalServerError().into()),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

pub fn confirm_registration(
    uuid: web::Path<Uuid>,
    data: web::Json<AuthData>,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::invitations::dsl::id as inv_id;
    use crate::schema::invitations::dsl::*;
    use crate::schema::users::dsl::email as u_email;
    use crate::schema::users::dsl::*;
    web::block(move || -> Result<LoggedUser, ServiceError> {
        let conn = pool.get().unwrap();
        let mut list_users = users.filter(u_email.eq(&data.email)).load::<User>(&conn)?;
        let mut list_inv = invitations
            .filter(inv_id.eq(&uuid.into_inner().to_string()))
            .load::<Invitation>(&conn)?;
        //does user exist
        if let Some(user) = list_users.pop() {
            //does user have an invitaion
            if let Some(inv) = list_inv.pop() {
                //verfy user credentials
                if let Ok(matching) = verify(&user.password, &data.password) {
                    //verification successful and invitation still valid
                    if matching
                        && (inv.expires_at > chrono::Local::now().naive_local()
                            && inv.resolved == 0)
                    {
                        println!("We are in!");
                        diesel::update(&user).set(active.eq(1)).execute(&conn)?;
                        diesel::update(&inv).set(resolved.eq(1)).execute(&conn)?;
                        return Ok(LoggedUser::from_user(&user));
                    } else {
                        println!(
                            "Passwords don't match or the invitation expired! matching: {:?}",
                            matching
                        );
                    }
                } else {
                    println!("Invitation not found");
                }
            } else {
                println!("User not found");
            }
        }
        println!("Unauthorized...");
        Err(ServiceError::Unauthorized)
    })
    .then(|res| match res {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn login(
    auth_data: web::Json<AuthData>,
    id: Identity,
    pool: web::Data<SqlPool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    use crate::schema::users::dsl::{email, users};
    web::block(move || -> Result<LoggedUser, ServiceError> {
        let conn = pool.get().unwrap();
        let mut items = users
            .filter(email.eq(&auth_data.email))
            .load::<User>(&conn)?;
        if let Some(user) = items.pop() {
            if let Ok(matching) = verify(&user.password, &auth_data.password) {
                if matching {
                    return Ok(LoggedUser::from_user(&user));
                }
            }
        }
        Err(ServiceError::Unauthorized)
    })
    .then(
        move |res: Result<LoggedUser, BlockingError<ServiceError>>| match res {
            Ok(user) => {
                let user_string = serde_json::to_string(&user).unwrap();
                id.remember(user_string);
                Ok(HttpResponse::Ok().json(user))
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        },
    )
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Ok().finish()
}

pub fn get_me(logged_user: LoggedUser) -> HttpResponse {
    HttpResponse::Ok().json(logged_user)
}
