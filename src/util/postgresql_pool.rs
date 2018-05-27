use std::env;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv;
use sapper::Key;

pub fn create_pg_pool() -> Arc<Pool<ConnectionManager<PgConnection>>> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::new(manager).expect("Failed to create pool.");
    Arc::new(pool)
}

pub struct Postgresql;

impl Key for Postgresql {
    type Value = Arc<Pool<ConnectionManager<PgConnection>>>;
}
