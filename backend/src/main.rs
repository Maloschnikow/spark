use std::{env, error::Error};

mod models;
mod routes;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let pool: sqlx::PgPool = sqlx::postgres::PgPool::connect(&database_url).await.expect("Database should be running");
    
    let app = routes::all_routes(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    return Ok(());
}
