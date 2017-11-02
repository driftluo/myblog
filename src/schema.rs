//table! {
//    posts (id) {
//        id -> Int4,
//        title -> Varchar,
//        content -> Text,
//        published -> Nullable<Bool>,
//        create_time -> Timestamp,
//        modify_time -> Timestamp,
//    }
//}

infer_schema!("dotenv:DATABASE_URL");
