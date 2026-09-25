use std::ops::Deref;

use langcities_lcdcdsl::component::UserAlias;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAliasDto(UserAlias);

impl Deref for UserAliasDto {
    type Target = UserAlias;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<UserAlias> for UserAliasDto {
    fn as_ref(&self) -> &UserAlias {
        self
    }
}

impl From<UserAlias> for UserAliasDto {
    fn from(value: UserAlias) -> Self {
        Self(value)
    }
}

impl Into<UserAlias> for UserAliasDto {
    fn into(self) -> UserAlias {
        self.0
    }
}
