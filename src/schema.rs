table! {
    users(id) {
        id -> Text,
        name -> Text,
        email -> Text,
        password -> Text,
        active -> Integer,
    }
}

table! {
    notes (id) {
        id -> Text,
        group_id -> Nullable<Text>,
        user_id -> Text,
        title -> Text,
        date_tag -> Timestamp,
        body -> Text,
        public -> Integer,
        pinned -> Integer,
    }
}

table! {
    groups (id) {
        id -> Text,
        created_at -> Timestamp,
        created_by -> Text,
        name -> Text,
    }
}

table! {
    group_links (id) {
        id -> Text,
        group_id -> Text,
        user_id -> Text,
    }
}


table! {
    invitations (id) {
        id -> Text,
        email -> Text,
        expires_at -> Timestamp,
        resolved -> Integer,
    }
}

allow_tables_to_appear_in_same_query! {
    users,
    notes,
    invitations,
    groups,
}
