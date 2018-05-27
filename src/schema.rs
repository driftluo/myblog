table!{
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

table!{
    articles (id) {
        id -> Uuid,
        title -> Varchar,
        raw_content -> Text,
        content -> Text,
        published -> Bool,
        create_time -> Timestamp,
        modify_time -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        account -> Varchar,
        github -> Nullable<Varchar>,
        password -> Varchar,
        salt -> Varchar,
        groups -> SmallInt,
        nickname -> Varchar,
        say -> Nullable<Varchar>,
        email -> Text,
        disabled -> SmallInt,
        create_time -> Timestamp,
    }
}

table!{
    tags (id) {
        id -> Uuid,
        tag -> Varchar,
    }
}

table!{
    comments (id) {
        id -> Uuid,
        comment -> Text,
        article_id -> Uuid,
        user_id -> Uuid,
        create_time -> Timestamp,
    }
}

table!{
    article_tag_relation (id) {
        id -> Uuid,
        tag_id -> Uuid,
        article_id -> Uuid,
    }
}

joinable!(article_tag_relation -> articles (article_id));
joinable!(article_tag_relation -> tags (tag_id));
joinable!(comments -> articles (article_id));
joinable!(comments -> users (user_id));

allow_tables_to_appear_in_same_query!(article_tag_relation, articles, comments, tags, users,);
