use crate::routes::auth::hash_password;
use crate::schema::{invitations, notes, users, groups, group_links};
use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

#[derive(
    Clone, Debug, Deserialize, Serialize, Insertable, Queryable, Identifiable, Associations,
)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub active: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Identifiable)]
#[table_name = "users"]
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
            password: hash_password(&user.password).unwrap(),
            active: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Insertable, Queryable, Identifiable)]
pub struct Invitation {
    pub id: String,
    pub email: String,
    pub expires_at: NaiveDateTime,
    pub resolved: i32,
}
impl Invitation {
    pub fn from_email(email: String) -> Self {
        Invitation {
            id: Uuid::new_v4().to_string(),
            email,
            expires_at: chrono::Local::now().naive_local() + chrono::Duration::hours(24),
            resolved: 0,
        }
    }

    pub fn from_user(user: &User) -> Self {
        Invitation {
            id: Uuid::new_v4().to_string(),
            email: user.email.clone(),
            expires_at: chrono::Local::now().naive_local() + chrono::Duration::hours(24),
            resolved: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Queryable, Associations, Insertable, Identifiable)]
#[belongs_to(Group)]
#[belongs_to(User)]
pub struct Note {
    pub id: String,
    pub group_id: Option<String>,
    pub user_id: String,
    pub title: String,
    pub date_tag: NaiveDateTime,
    pub body: String,
    pub public: i32,
    pub pinned: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewNote {
    pub title: String,
    pub group_id: Option<String>,
    pub date_tag: String,
    pub body: String,
    pub public: i32,
    pub pinned: i32,
}


#[derive(Clone, Debug, AsChangeset, Deserialize)]
#[table_name = "notes"]
pub struct NotePatch {
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub title: Option<String>,
    pub date_tag: Option<String>,
    pub body: Option<String>,
    pub public: Option<i32>,
    pub pinned: Option<i32>,
}

impl Note {
    pub fn from(note: NewNote, user: LoggedUser) -> Self {
        Note {
            id: Uuid::new_v4().to_string(),
            group_id: note.group_id,
            user_id: user.id,
            title: note.title,
            date_tag:  NaiveDateTime::parse_from_str(&note.date_tag, "%Y-%m-%d %H:%M:%S").unwrap(),
            body: note.body,
            public: note.public,
            pinned: note.pinned,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Insertable, Queryable, Identifiable)]
pub struct Group {
    pub id: String,
    pub created_at: NaiveDateTime,
    pub created_by: String,
    pub name: String,
    pub color: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct NewGroup {
    pub color: String,
    pub name: String,
}

impl Group {
    pub fn from(group: NewGroup, user: LoggedUser) -> Self {
        let mut now = Utc::now().naive_utc().to_string();
        now.truncate(19);
        Group {
            id: Uuid::new_v4().to_string(),
            created_at: NaiveDateTime::parse_from_str(&now, "%Y-%m-%d %H:%M:%S").unwrap(),
            created_by: user.id,
            name: group.name,
            color: group.color,
        }
    }
}


#[derive(Clone, Debug, Deserialize, Serialize, Associations, Insertable, Queryable, Identifiable)]
//#[belongs_to(User)]
#[belongs_to(LoggedUser, foreign_key="user_id")]
#[belongs_to(Group)]
#[table_name = "group_links"]
pub struct GroupLink {
    pub id: String,
    pub user_id: String,
    pub group_id: String,
}

impl GroupLink {
    pub fn from(group: &Group, user: &LoggedUser) -> Self {
        GroupLink {
            id: Uuid::new_v4().to_string(),
            group_id: group.id.clone(),
            user_id: user.id.clone(),
        }
    }
}