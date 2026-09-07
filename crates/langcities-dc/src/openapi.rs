use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        crate::route::v1::token::validate_token,
        crate::route::v1::users::get_user,
        crate::route::v1::vernaculars::get_vernacular
    ),
    components(schemas(
        crate::dto::vernaculars::VernacularAliasDto
    )),
    security(("bearer" = []))
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn vernacular_alias_parameter_references_registered_string_schema() {
        let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let parameter = &document["paths"]["/v1/vernaculars/{alias}"]["get"]["parameters"][0];

        assert_eq!(parameter["name"], json!("alias"));
        assert_eq!(
            parameter["schema"]["$ref"],
            json!("#/components/schemas/VernacularAliasDto")
        );
        assert_eq!(
            document["components"]["schemas"]["VernacularAliasDto"]["type"],
            json!("string")
        );
    }
}
