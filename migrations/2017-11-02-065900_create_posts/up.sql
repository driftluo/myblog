-- Your SQL goes here

create extension pgcrypto;

Create table articles (
    id uuid primary key default gen_random_uuid(),
    title varchar not null,
    raw_content text not null,
    content text not null,
    published bool not null default false,
    create_time timestamp not null default current_timestamp,
    modify_time timestamp not null default current_timestamp
);

-- Function

CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modify_time = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- trigger

Create Trigger update_posts_modify_time before update
on articles for each row execute procedure update_modified_column();
