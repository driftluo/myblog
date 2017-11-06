-- Your SQL goes here

Create table tags (
  id serial primary key,
  tag varchar not null
);

Create table article_tag_relation (
  id serial primary key,
  tag_id integer not null references tags (id),
  article_id integer not null references articles (id)
);

 Create or replace view article_with_tag as
 select a.id, a.title, a.content, a.published, array_agg(c.id) as tags_id, array_agg(c.tag) as tags, a.create_time, a.modify_time
 from articles a
 left join article_tag_relation b on a.id=b.article_id
 left join tags c on b.tag_id=c.id
 group by a.id, a.title, a.content, a.published, a.create_time, a.modify_time
