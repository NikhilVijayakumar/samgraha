use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Urn(String);

impl Urn {
    pub fn new(domain: &str, slug: &str) -> Self {
        Self(format!("{}/{}", domain, slug))
    }

    pub fn for_document(domain: &str, slug: &str) -> Self {
        let slug = slug.to_lowercase().replace(' ', "-").replace('/', "-");
        Self(format!("{}/{}", domain, slug))
    }

    pub fn for_item(&self, fragment: &str) -> Self {
        Self(format!("{}/{}", self.0, fragment))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn domain(&self) -> Option<&str> {
        self.0.split('/').next()
    }

    pub fn slug(&self) -> Option<&str> {
        self.0.split('/').nth(1)
    }

    pub fn fragment(&self) -> Option<&str> {
        self.0.split('/').nth(2)
    }
}

impl fmt::Display for Urn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Urn {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') {
            Ok(Self(s.to_string()))
        } else {
            Err(format!("Invalid URN format: '{}'", s))
        }
    }
}

impl From<&str> for Urn {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Urn {
    fn from(s: String) -> Self {
        Self(s)
    }
}
