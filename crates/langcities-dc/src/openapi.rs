use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(crate::route::v1::token::validate_token))]
pub struct ApiDoc;
