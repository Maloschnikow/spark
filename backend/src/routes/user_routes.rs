use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use clorinde::{
    deadpool_postgres::Pool,
    queries::users::{get_all_users, insert_user},
};

use crate::api_types::User;

pub fn routes() -> Router<Pool> {
    Router::new()
        .route("/users", get(get_users))
        .route("/user", post(post_user))
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = StatusCode::OK, description = "Return a list of all users", body = Vec<User>),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Returns the error message", body = String)
    )
)]
#[axum::debug_handler]
async fn get_users(State(pool): State<Pool>) -> (StatusCode, Result<Json<Vec<User>>, String>) {
    //let query = sqlx::query_as::<Postgres, User>("SELECT id, username FROM users");

    let pool_result = pool.get().await;
    if let Err(e) = pool_result {
        return (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string()));
    }
    let client = pool_result.unwrap();

    let result = get_all_users().bind(&client).all().await;
    if result.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Err(result.unwrap_err().to_string()),
        );
    }
    let vec = result.unwrap();
    let user_vec = vec
        .iter()
        .map(|row| User {
            id: row.id,
            username: row.username.clone(),
        })
        .collect();

    return (StatusCode::OK, Ok(Json(user_vec)));
}

#[utoipa::path(
    post,
    path = "/user",
    request_body(
        content = String,
        description = "Username of the user to be created"
    ),
    responses(
        (status = StatusCode::CREATED, description = "Successfully created user with username", body = User),
        (status = StatusCode::CONFLICT, description = "User already exists", body = String),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Returns the error message", body = String)
    )
)]
#[axum::debug_handler]
async fn post_user(
    State(pool): State<Pool>,
    username: String,
) -> (StatusCode, Result<Json<User>, String>) {
    let pool_result = pool.get().await;
    if let Err(e) = pool_result {
        return (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string()));
    }
    let client = pool_result.unwrap();

    let result = insert_user().bind(&client, &username).one().await;
    match result {
        Ok(row) => {
            let inserted_user = User {
                id: row.id,
                username: row.username,
            };
            return (StatusCode::CREATED, Ok(Json(inserted_user)));
        }
        Err(e) => {
            if let Some(some_eb_error) = e.as_db_error() {
                // Check for PostgreSQL unique violation error code
                if some_eb_error.code().code() == "23505" {
                    return (StatusCode::CONFLICT, Err("User already exists".to_string()));
                }
            }
            return (StatusCode::INTERNAL_SERVER_ERROR, Err(e.to_string()));
        }
    }
}
