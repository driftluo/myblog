use std::env;
use std::sync::Arc;
use std::default::Default;

use dotenv;
use typemap::Key;
use r2d2;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use diesel::pg::PgConnection;

pub fn create_pg_pool() -> Arc<Pool<ConnectionManager<PgConnection>>> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(config, manager).expect("Failed to create pool.");
    Arc::new(pool)
}

pub struct Postgresql;

impl Key for Postgresql {
    type Value = Arc<Pool<ConnectionManager<PgConnection>>>;
}
