use super::super::articles::dsl::articles as all_articles;
use super::super::{ articles, article_with_tag };
use super::super::article_with_tag::dsl::article_with_tag as all_article_with_tag;
use super::super::{ markdown_render };
use super::{ Relations, RelationTag };

use chrono::NaiveDateTime;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl,
              SelectDsl, OrderDsl, LimitDsl, OffsetDsl, PgConnection };
use diesel::expression::sql;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticlesWithTag {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub tags_id: Vec<Option<Uuid>>,
    pub tags: Vec<Option<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl ArticlesWithTag {
    pub fn delete_with_id(conn: &PgConnection, id: Uuid) -> Result<usize, String> {
        Relations::delete_all(conn, id, "article");
        let res = diesel::delete(all_articles.filter(articles::id.eq(id)))
        .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn query_article(conn: &PgConnection, id: Uuid, admin: bool) -> Result<ArticlesWithTag, String> {
        let res = if admin {
            all_article_with_tag.filter(article_with_tag::id.eq(id))
                .get_result::<RawArticlesWithTag>(conn)
        } else {
            all_article_with_tag.filter(article_with_tag::id.eq(id))
                .filter(article_with_tag::published.eq(true))
                .get_result::<RawArticlesWithTag>(conn)
        };

        match res {
            Ok(data) => {
                    Ok(data.into_html())
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn query_raw_article(conn: &PgConnection, id: Uuid) -> Result<ArticlesWithTag, String> {
        let res = all_article_with_tag.filter(article_with_tag::id.eq(id))
            .get_result::<RawArticlesWithTag>(conn);
        match res {
            Ok(data) => {
                Ok(data.into_markdown())
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn publish_article(conn: &PgConnection, data: ModifyPublish) -> Result<usize, String> {
        let res = diesel::update(all_articles.filter(articles::id.eq(data.id)))
            .set(articles::published.eq(data.publish))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct ArticleList {
    pub id: Uuid,
    pub title: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime
}

impl ArticleList {
    pub fn query_list_article(conn: &PgConnection, limit: i64, offset: i64, admin: bool) -> Result<Vec<ArticleList>, String> {
        let res = if admin {
                all_articles
                .select((articles::id, articles::title, articles::published, articles::create_time, articles::modify_time))
                .order(articles::create_time.desc())
                .limit(limit)
                .offset(offset)
                .load::<ArticleList>(conn)
        } else {
            all_articles
                .select((articles::id, articles::title, articles::published, articles::create_time, articles::modify_time))
                .filter(articles::published.eq(true))
                .order(articles::create_time.desc())
                .limit(limit)
                .offset(offset)
                .load::<ArticleList>(conn)
        };

        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn query_with_tag(conn: &PgConnection, tag_id: Uuid) -> Result<Vec<ArticleList>, String> {
        let raw_sql = format!("select id, title, published, create_time, modify_time from article_with_tag where ('{}' = any(tags_id)) and published = true order by create_time desc", tag_id);
        let query = sql::<(diesel::types::Uuid, diesel::types::Text, diesel::types::Bool, diesel::types::Timestamp, diesel::types::Timestamp)>(&raw_sql);
        let res = query.load::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "articles"]
struct InsertArticle {
    title: String,
    raw_content: String,
    content: String,
}

impl InsertArticle {
    fn new(title: String, raw_content: String) -> Self {
        let content = markdown_render(&raw_content);
        InsertArticle {
            title,
            raw_content,
            content
        }
    }

    fn insert(&self, conn: &PgConnection) -> Articles {
        diesel::insert(self)
            .into(articles::table)
            .get_result::<Articles>(conn).unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewArticle {
    pub title: String,
    pub raw_content: String,
    pub exist_tags: Option<Vec<Uuid>>,
    pub new_tags: Option<Vec<String>>,
}

impl NewArticle {
    pub fn insert(self, conn: &PgConnection) -> bool {
        let article = self.convert_insert_article().insert(conn);
        if self.new_tags.is_some() || self.exist_tags.is_some() {
            return RelationTag::new(article.id, self.new_tags, self.exist_tags).insert_all(conn)
        } else {
            return true
        }
    }

    fn convert_insert_article(&self) -> InsertArticle {
        InsertArticle::new(self.title.to_owned(), self.raw_content.to_owned())
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    id: Uuid,
    title: String,
    raw_content: String,
    new_choice_already_exists_tags: Option<Vec<Uuid>>,
    deselect_tags: Option<Vec<Uuid>>,
    new_tags: Option<Vec<String>>
}

impl EditArticle {
    pub fn edit_article(self, conn: &PgConnection) -> Result<usize, String> {
        let res = diesel::update(all_articles.filter(articles::id.eq(self.id)))
            .set((articles::title.eq(self.title),
                  articles::content.eq(markdown_render(&self.raw_content)), articles::raw_content.eq(self.raw_content)
            ))
            .execute(conn);
        if self.new_tags.is_some() || self.new_choice_already_exists_tags.is_some() {
            RelationTag::new(self.id, self.new_tags, self.new_choice_already_exists_tags).insert_all(conn);
        }
        if self.deselect_tags.is_some() {
            for i in self.deselect_tags.unwrap() {
                Relations::new(self.id, i).delete_relation(conn);
            }
        }
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ModifyPublish {
    id: Uuid,
    publish: bool
}

#[derive(Queryable, Debug, Clone)]
struct RawArticlesWithTag {
    pub id: Uuid,
    pub title: String,
    pub raw_content: String,
    pub content: String,
    pub published: bool,
    pub tags_id: Vec<Option<Uuid>>,
    pub tags: Vec<Option<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl RawArticlesWithTag {
    fn into_markdown(self) -> ArticlesWithTag {
        ArticlesWithTag {
            id: self.id,
            title: self.title,
            content: self.raw_content,
            published: self.published,
            tags_id: self.tags_id,
            tags: self.tags,
            create_time: self.create_time,
            modify_time: self.modify_time
        }
    }

    fn into_html(self) -> ArticlesWithTag {
        ArticlesWithTag {
            id: self.id,
            title: self.title,
            content: self.content,
            published: self.published,
            tags_id: self.tags_id,
            tags: self.tags,
            create_time: self.create_time,
            modify_time: self.modify_time
        }
    }
}

#[derive(Queryable, Debug, Clone)]
struct Articles {
    pub id: Uuid,
    pub title: String,
    pub raw_content: String,
    pub content: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

#[derive(Queryable, Debug, Clone, Deserialize, Serialize)]
pub struct PublishedStatistics {
    pub dimension: String,
    pub quantity: i64,
}

impl PublishedStatistics {
    pub fn statistics_published_frequency_by_month(conn: &PgConnection) -> Result<Vec<Self>, String> {
        let raw_sql = "select to_char(create_time, 'yyyy-mm') as dimension, count(*) as quantity from articles group by dimension order by dimension;";
        let query = sql::<(diesel::types::Text, diesel::types::BigInt)>(raw_sql);
        let res = query.load::<Self>(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }
}
