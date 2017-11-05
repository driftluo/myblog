-- Your SQL goes here

Create table users (
  id serial primary key,
  account VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  salt VARCHAR NOT NULL,
  nickname VARCHAR NOT NULL,
  say VARCHAR,
  email character varying(128) UNIQUE NOT NULL,
  create_time timestamp NOT NULL default current_timestamp
);
