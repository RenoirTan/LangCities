use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(crate::route::v1::login::password_login))]
pub struct ApiDoc;
