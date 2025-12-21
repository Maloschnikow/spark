use std::{env, error::Error};

use clorinde::{
    deadpool_postgres::{Config, Runtime},
    tokio_postgres::NoTls,
};

mod api_types;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO do config right (needs to work with ip and socket files; best would be if database_url could be used)
    // let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let host: String = env::var("PG_HOST").expect("PG_HOST should be set");
    let user: String = env::var("CUSTOM_PGUSER").expect("CUSTOM_PGUSER should be set");
    let password: String = env::var("PGPASSWORD").expect("PGPASSWORD should be set");
    let database_name: String = env::var("PGDATABASE").expect("PGDATABASE should be set");

    let mut cfg = Config::new();
    cfg.host = Some(host);
    cfg.user = Some(user);
    cfg.password = Some(password);
    cfg.dbname = Some(database_name);

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let app = routes::all_routes(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    return Ok(());
}
