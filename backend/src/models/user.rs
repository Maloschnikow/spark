use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug,FromRow, ToSchema, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}
