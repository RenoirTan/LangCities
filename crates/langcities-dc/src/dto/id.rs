use std::ops::Deref;

use langcities_lcdcdsl::component::Alias;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAliasDto(Alias);

impl Deref for UserAliasDto {
    type Target = Alias;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Alias> for UserAliasDto {
    fn as_ref(&self) -> &Alias {
        self
    }
}

impl From<Alias> for UserAliasDto {
    fn from(value: Alias) -> Self {
        Self(value)
    }
}

impl Into<Alias> for UserAliasDto {
    fn into(self) -> Alias {
        self.0
    }
}
