use crate::schema::{
    notes
};
use chrono::{ NaiveDateTime };
use uuid::Uuid;


pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub active: i8,
}

pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

pub struct LoggedUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl LoggedUser {
    pub fn from(user: User) -> Self {
        LoggedUser {
            id: user.id,
            name: user.name,
            email: user.email,
        }
    }
}

impl User {
    pub fn from(user: NewUser) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            name: user.name,
            email: user.email,
            password: user.password,
            active: 0,
        }
    }
}




#[derive(Clone, Debug, Serialize, Queryable, Insertable)]
pub struct Note {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub date_tag: NaiveDateTime,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewNote {
    pub user_id: String,
    pub title: String,
    pub date_tag: NaiveDateTime,
    pub body: String,
}


impl Note {
    pub fn from(note: NewNote) -> Self {
        Note {
            id: Uuid::new_v4().to_string(),
            user_id: note.user_id,
            title: note.title,
            date_tag: note.date_tag,
            body: note.body,
        }
    }
}