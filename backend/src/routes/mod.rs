pub mod user_routes;

use axum::Router;
use clorinde::deadpool_postgres::Pool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api_types::User;

#[derive(OpenApi)]
#[openapi(
    paths(
        user_routes::get_users,
        user_routes::post_user
    ),
    components(
        schemas(User)
    ),
    tags(
        (name = "users", description = "User management")
    )
)]
struct ApiDoc;
pub fn all_routes(pool: Pool) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/doc").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(user_routes::routes())
        .with_state(pool)
    //.merge(other_routes::routes())
}
