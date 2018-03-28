-- Your SQL goes here

Create table users (
  id uuid primary key default gen_random_uuid(),
  account VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  salt VARCHAR NOT NULL,
  groups smallint not null default 1,
  nickname VARCHAR NOT NULL,
  say VARCHAR,
  email character varying(128) UNIQUE NOT NULL,
  disabled smallint not null default 0,
  create_time timestamp NOT NULL default current_timestamp,
  github varchar unique
);

Create index user_account on users (account);
Create index github on users (github);

insert into users (account, password, salt, groups, nickname, email, github) values
('admin', '325c162157dea106ce5bacc705c4929e4ec526a0290bfaba2dcbbf18103c7c2b', 'MKsiaw', 0, '漂流', '441594700@qq.com', 'https://github.com/driftluo');
