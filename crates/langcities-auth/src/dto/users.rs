use langcities_common_server::dto::users::AuthUserDto;

use crate::entity::users;

impl From<users::Model> for AuthUserDto {
    fn from(value: users::Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use langcities_common_server::dto::users::{AuthUsersDto, ManyUserAliasDto};
    use serde_json::json;

    #[test]
    fn shared_user_response_preserves_the_public_json_contract() {
        let model = users::Model {
            id: 42,
            username: "alice".into(),
            password_hash: Some("private-hash".into()),
            created_at: "2026-01-01T00:00:00Z".parse().unwrap(),
            updated_at: "2026-01-02T00:00:00Z".parse().unwrap(),
        };
        let dto: AuthUsersDto = [model].into_iter().collect();
        let expected = json!({"users": [{
            "id": 42,
            "username": "alice",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z"
        }]});
        assert_eq!(serde_json::to_value(&dto).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<AuthUsersDto>(expected).unwrap(),
            dto
        );
        assert!(
            serde_json::from_value::<AuthUsersDto>(json!({}))
                .unwrap()
                .users
                .is_empty()
        );
    }

    #[test]
    fn shared_alias_query_accepts_repeated_parameters() {
        use axum::http::Uri;
        use axum_extra::extract::Query;

        let uri: Uri = "/v1/users?aliases=alice&aliases=bob".parse().unwrap();
        let Query(query) = Query::<ManyUserAliasDto>::try_from_uri(&uri).unwrap();
        assert_eq!(query.aliases.len(), 2);
        assert_eq!(query.aliases[0].0.to_string(), "alice");
        assert_eq!(query.aliases[1].0.to_string(), "bob");

        let empty: Uri = "/v1/users".parse().unwrap();
        assert!(
            Query::<ManyUserAliasDto>::try_from_uri(&empty)
                .unwrap()
                .aliases
                .is_empty()
        );
    }
}
