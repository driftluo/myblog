infer_schema!("dotenv:DATABASE_URL");

table! {
    article_with_tag (id) {
        id -> Int4,
        title -> Varchar,
        raw_content -> Text,
        content -> Text,
        published -> Bool,
        tags_id -> Array<Nullable<Int4>>,
        tags -> Array<Nullable<Text>>,
        create_time -> Timestamp,
        modify_time -> Timestamp,
    }
}
