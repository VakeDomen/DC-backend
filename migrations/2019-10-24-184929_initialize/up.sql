-- Your SQL goes here
create table users
(
    id          varchar not null primary key,
    name        varchar not null,
    email       varchar not null,
    password    varchar not null,
    active      varchar not null
);

create table notes
(
    id          varchar not null primary key,
    user_id     varchar not null,
    title       varchar not null,
    date_tag    datetime not null,
    body        varchar not null
);