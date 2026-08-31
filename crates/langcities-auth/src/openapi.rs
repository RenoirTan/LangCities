use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    crate::route::v1::login::password_login,
    crate::route::v1::token::issue_generic_access_token,
    crate::route::v1::token::issue_dc_access_token
))]
pub struct ApiDoc;
