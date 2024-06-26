use utoipa::OpenApi;

use super::IdentifierQuery;
use axum::extract::Query;


#[utoipa::path(
    get,
    path = "/api/container/profile_full",
    params(
        IdentifierQuery
    ),
    responses(
        (status = 200, description = "Container")
    ),
    tag = "universal.container",
)]
pub fn profile_handler_dummy(
    _query: Query<IdentifierQuery>,) {
}


#[utoipa::path(
    get,
    path = "/api/container/list",
    responses(
        (status = 200, description = "System")
    ),
    tag = "universal.container",
)]
pub fn list_dummy() {
}

#[derive(OpenApi)]
#[openapi(
    paths(
        profile_handler_dummy,
        list_dummy,
    ),
    components(schemas(
    ))
)]
pub struct ApiDoc;