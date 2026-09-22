use std::time::Duration;

use langcities_common::error::Error;
use langcities_config::error::{LcConfigError, LcConfigErrorTrait};
use reqwest::{Client, Url};

use crate::dto::users::{AuthUserDto, AuthUsersDto};

/// Validates an auth service origin without a path prefix, credentials, query, or fragment.
pub fn validate_base_url(base_url: &Url) -> Result<(), LcConfigError> {
    if !matches!(base_url.scheme(), "http" | "https")
        || base_url.host_str().is_none()
        || base_url.path() != "/"
        || base_url.query().is_some()
        || base_url.fragment().is_some()
        || !base_url.username().is_empty()
        || base_url.password().is_some()
    {
        return Err(LcConfigError::bad_parse(
            "auth_base_url must be an HTTP(S) origin, e.g. http://langcities-auth:8000",
        ));
    }
    Ok(())
}

/// A reusable HTTP client for the auth service. Clones share the connection pool.
#[derive(Clone, Debug)]
pub struct AuthClient {
    http: Client,
    users_url: Url,
}

impl AuthClient {
    /// Creates a client with a 2-second connection timeout and a 5-second total timeout.
    /// The users endpoint is resolved at `/v1/users` on the supplied service origin.
    /// Returns an error if the origin fails [`validate_base_url`].
    pub fn new(base_url: &Url) -> Result<Self, Error> {
        validate_base_url(base_url)?;
        let users_url = base_url.join("/v1/users")?;
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { http, users_url })
    }

    /// Fetches matching users using repeated `aliases` query parameters.
    /// An empty slice returns no users without making a request. Missing aliases
    /// are omitted from the response; HTTP and deserialization errors are propagated.
    pub async fn fetch_users(&self, aliases: &[&str]) -> Result<Vec<AuthUserDto>, reqwest::Error> {
        if aliases.is_empty() {
            return Ok(Vec::new());
        }
        let query: Vec<_> = aliases.iter().map(|a| ("aliases", a)).collect();
        let response = self
            .http
            .get(self.users_url.clone())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<AuthUsersDto>()
            .await?;
        Ok(response.users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_requires_an_http_origin() {
        for value in [
            "ftp://localhost",
            "mailto:user@example.com",
            "http://localhost/auth",
            "http://localhost?query=1",
            "http://localhost#fragment",
            "http://user@localhost",
            "http://:password@localhost",
        ] {
            let url = Url::parse(value).unwrap();
            assert!(AuthClient::new(&url).is_err(), "accepted {value}");
        }
        for value in ["http://localhost:8000", "https://auth.example.com/"] {
            let url = Url::parse(value).unwrap();
            let client = AuthClient::new(&url).unwrap();
            assert_eq!(client.users_url, url.join("/v1/users").unwrap());
        }
    }
}
