use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sqlx::{PgPool, Postgres};

use crate::models::user::User;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/users", get(get_users))
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = StatusCode::OK, description = "Return a list of all users", body = Vec<User>),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Returns the error message", body = String)
    )
)]
async fn get_users(State(pool): State<PgPool>) -> (StatusCode, Result<Json<Vec<User>>, String>) {
    let query = sqlx::query_as::<Postgres, User>("SELECT id, username FROM users");

    let result = query.fetch_all(&pool).await;
    if result.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Err(result.unwrap_err().to_string()));
    }
    return (StatusCode::OK, Ok(Json(result.unwrap())));
}
