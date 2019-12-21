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
    group_id    varchar null,
    user_id     varchar not null,
    title       varchar not null,
    date_tag    datetime not null,
    body        varchar not null,
    public      int not null,
    pinned      int not null
);

create table groups
(
  id            varchar not null primary key,
  created_at    datetime not null,
  created_by    varchar not null,
  name          varchar not null
);


create table invitations
(
  id          varchar not null primary key,
  email       varchar not null,
  expires_at  datetime not null,
  resolved    int not null
);
