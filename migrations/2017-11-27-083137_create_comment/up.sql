-- Your SQL goes here

Create table comments (
    id uuid primary key default gen_random_uuid(),
    comment text not null,
    article_id uuid not null references articles (id),
    user_id uuid not null references users (id),
    user_nickname VARCHAR NOT NULL,
    re_user_id uuid references users (id),
    re_user_nickname varchar
);
