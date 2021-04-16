-- Add migration script here

create extension pgcrypto;

-- articles

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

-- user

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

-- tags

Create table tags (
      id uuid primary key default gen_random_uuid(),
      tag varchar not null
);

Create table article_tag_relation (
      id uuid primary key default gen_random_uuid(),
      tag_id uuid not null references tags (id),
      article_id uuid not null references articles (id)
);

Create or replace view article_with_tag as
select a.id, a.title, a.raw_content, a.content, a.published, array_agg(c.id) as tags_id, array_agg(c.tag) as tags, a.create_time, a.modify_time
from articles a
         left join article_tag_relation b on a.id=b.article_id
         left join tags c on b.tag_id=c.id
group by a.id, a.title, a.content, a.published, a.create_time, a.modify_time;

-- comments

Create table comments (
      id uuid primary key default gen_random_uuid(),
      comment text not null,
      article_id uuid not null references articles (id),
      user_id uuid not null references users (id),
      create_time timestamp NOT NULL default current_timestamp
);
