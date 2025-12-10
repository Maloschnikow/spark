-- UP
BEGIN;

create table users (
    id serial primary key,
    username text unique not null
);

COMMIT;