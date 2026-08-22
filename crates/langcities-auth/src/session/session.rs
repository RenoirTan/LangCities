use std::ops::{Deref, DerefMut};

use tower_sessions::Session;

#[derive(Clone, Debug)]
pub struct SessionWrapper {
    inner: Session,
}

impl SessionWrapper {
    pub const USER_ID_KEY: &'static str = "user_id";

    pub fn new(session: impl Into<Session>) -> Self {
        let session = session.into();
        Self { inner: session }
    }

    pub async fn get_user_id(&self) -> Option<i64> {
        self.inner.get(Self::USER_ID_KEY).await.ok().flatten()
    }
}

impl Deref for SessionWrapper {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SessionWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AsRef<Session> for SessionWrapper {
    fn as_ref(&self) -> &Session {
        &*self
    }
}

impl AsMut<Session> for SessionWrapper {
    fn as_mut(&mut self) -> &mut Session {
        &mut *self
    }
}

impl From<Session> for SessionWrapper {
    fn from(value: Session) -> Self {
        Self::new(value)
    }
}
