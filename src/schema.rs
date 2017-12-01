infer_schema!("dotenv:DATABASE_URL");

table! {
    article_with_tag (id) {
        id -> Uuid,
        title -> Varchar,
        raw_content -> Text,
        content -> Text,
        published -> Bool,
        tags_id -> Array<Nullable<Uuid>>,
        tags -> Array<Nullable<Text>>,
        create_time -> Timestamp,
        modify_time -> Timestamp,
    }
}
