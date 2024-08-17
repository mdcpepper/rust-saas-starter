CREATE TABLE IF NOT EXISTS users
(
    id uuid primary key not null,
    email text not null unique,
    password text not null,
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null
)
