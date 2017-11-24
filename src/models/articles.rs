use super::super::articles::dsl::articles as all_articles;
use super::super::{ articles, article_with_tag };
use super::super::article_with_tag::dsl::article_with_tag as all_article_with_tag;
use super::super::{ markdown_render };
use super::Relations;

use chrono::NaiveDateTime;
use diesel;
use diesel::{ FilterDsl, ExpressionMethods, ExecuteDsl, LoadDsl,
              SelectDsl, OrderDsl, LimitDsl, OffsetDsl, PgConnection };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Articles {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub tags_id: Vec<Option<i32>>,
    pub tags: Vec<Option<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl Articles {
    pub fn delete_with_id(conn: &PgConnection, id: i32) -> Result<usize, String> {
        Relations::delete_all(conn, id, "article");
        let res = diesel::delete(all_articles.filter(articles::id.eq(id)))
        .execute(conn);
        match res {
            Ok(data) => Ok(data),
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn query_article(conn: &PgConnection, id: i32, admin: bool) -> Result<Articles, String> {
        let res = if admin {
            all_article_with_tag.filter(article_with_tag::id.eq(id))
                .get_result::<RawArticles>(conn)
        } else {
            all_article_with_tag.filter(article_with_tag::id.eq(id))
                .filter(article_with_tag::published.eq(true))
                .get_result::<RawArticles>(conn)
        };

        match res {
            Ok(data) => {
                    Ok(data.into_html())
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn query_raw_article(conn: &PgConnection, id: i32) -> Result<Articles, String> {
        let res = all_article_with_tag.filter(article_with_tag::id.eq(id))
            .get_result::<RawArticles>(conn);
        match res {
            Ok(data) => {
                Ok(data.into_markdown())
            },
            Err(err) => Err(format!("{}", err))
        }
    }

    pub fn edit_article(conn: &PgConnection, data: EditArticle) -> Result<usize, String> {
        let res = diesel::update(all_articles.filter(articles::id.eq(data.id)))
            .set((articles::title.eq(data.title),
                  articles::content.eq(markdown_render(&data.raw_content)), articles::raw_content.eq(data.raw_content)
                  ))
            .execute(conn);
        match res {
            Ok(data) => Ok(data),
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
    pub id: i32,
    pub title: String,
    pub published: bool,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
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

    fn insert(&self, conn: &PgConnection) -> bool {
        diesel::insert(self)
            .into(articles::table)
            .execute(conn)
            .is_ok()
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewArticle {
    pub title: String,
    pub raw_content: String,
}

impl NewArticle {
    pub fn insert(self, conn: &PgConnection) -> bool {
        self.into_insert_article().insert(conn)
    }

    fn into_insert_article(self) -> InsertArticle {
        InsertArticle::new(self.title, self.raw_content)
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    id: i32,
    title: String,
    raw_content: String
}

#[derive(Deserialize, Serialize)]
pub struct ModifyPublish {
    id: i32,
    publish: bool
}

#[derive(Queryable, Debug, Clone)]
struct RawArticles {
    pub id: i32,
    pub title: String,
    pub raw_content: String,
    pub content: String,
    pub published: bool,
    pub tags_id: Vec<Option<i32>>,
    pub tags: Vec<Option<String>>,
    pub create_time: NaiveDateTime,
    pub modify_time: NaiveDateTime,
}

impl RawArticles {
    fn into_markdown(self) -> Articles {
        Articles {
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

    fn into_html(self) -> Articles {
        Articles {
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
