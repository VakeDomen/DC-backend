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
    notes(id) {
        id -> Text,
        user_id -> Text,
        title -> Text,
        date_tag -> Timestamp,
        body -> Text,
    }
}

allow_tables_to_appear_in_same_query! {
    users,
    notes
}