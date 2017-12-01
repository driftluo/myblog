-- Your SQL goes here

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
