-- Your SQL goes here

Create table comments (
    id uuid primary key default gen_random_uuid(),
    comment text not null,
    article_id uuid not null references articles (id),
    user_id uuid not null references users (id),
    create_time timestamp NOT NULL default current_timestamp
);
