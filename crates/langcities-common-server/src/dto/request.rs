use langcities_lcdcdsl::component::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    pub caller_id: Option<Id>,
    pub access_kind: RequestAccessKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestAccessKind {
    NormalUser,
    System,
}
