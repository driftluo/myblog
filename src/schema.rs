infer_schema!("dotenv:DATABASE_URL");

table! {
    article_with_tag (id) {
        id -> Int4,
        title -> Varchar,
        content -> Text,
        published -> Bool,
        tags -> Nullable<Array<Text>>,
        create_time -> Timestamp,
        modify_time -> Timestamp,
    }
}
