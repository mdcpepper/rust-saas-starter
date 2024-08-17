create table if not exists users
(
    id uuid primary key not null,
    email text not null unique,
);
