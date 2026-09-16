use langcities_common_server::dto::users::{
    AuthUserDto, AuthUsersDto, ManyUserAliasDto, UserAliasDto,
};
use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto(paths = "./crates/langcities-auth/src")]
#[derive(OpenApi)]
#[openapi(components(schemas(AuthUserDto, AuthUsersDto, UserAliasDto, ManyUserAliasDto)))]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn user_endpoints_reference_registered_shared_schemas() {
        let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let schemas = &document["components"]["schemas"];
        for (path, method, schema) in [
            ("/v1/users/{alias}", "get", "AuthUserDto"),
            ("/v1/users", "get", "AuthUsersDto"),
            ("/v1/users", "delete", "AuthUserDto"),
            ("/v1/register", "post", "AuthUserDto"),
        ] {
            let response = &document["paths"][path][method]["responses"]["200"]["content"]["application/json"]
                ["schema"];
            assert_eq!(
                response["$ref"],
                json!(format!("#/components/schemas/{schema}"))
            );
            assert!(schemas.get(schema).is_some(), "missing schema {schema}");
        }
        assert_eq!(
            schemas["AuthUsersDto"]["properties"]["users"]["items"]["$ref"],
            json!("#/components/schemas/AuthUserDto")
        );
        for field in ["created_at", "updated_at"] {
            assert_eq!(
                schemas["AuthUserDto"]["properties"][field]["format"],
                json!("date-time")
            );
            assert!(
                schemas["AuthUserDto"]["required"]
                    .as_array()
                    .unwrap()
                    .contains(&json!(field))
            );
        }
        assert_eq!(schemas["UserAliasDto"]["type"], json!("string"));
        assert!(schemas.get("ManyUserAliasDto").is_some());
        assert!(schemas.get("UserDto").is_none());
        assert!(schemas.get("ManyUsersDto").is_none());

        let aliases = &document["paths"]["/v1/users"]["get"]["parameters"][0];
        assert_eq!(aliases["name"], json!("aliases"));
        assert_eq!(aliases["style"], json!("form"));
        assert_eq!(aliases["explode"], json!(true));
    }
}
