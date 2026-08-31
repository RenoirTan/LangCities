#[derive(Clone, Debug)]
pub enum Microservice {
    Generic,
    Dc,
}

impl Microservice {
    pub fn allowed_audiences(&self) -> &'static [&'static str] {
        match self {
            Self::Generic => &["generic"],
            Self::Dc => &["dc"],
        }
    }

    pub fn first_allowed_audience(&self) -> &'static str {
        self.allowed_audiences()[0]
    }
}
