use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto(paths = "./crates/langcities-auth/src")]
#[derive(OpenApi)]
#[openapi]
pub struct ApiDoc;
