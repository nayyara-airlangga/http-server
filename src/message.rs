use std::{error::Error, fmt::Display, str::FromStr};

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Invalid,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid HTTP Method")
    }
}

impl Error for HttpMethod {}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &str {
        match *self {
            Self::Get => "GET",
            Self::Invalid => "INVALID",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = Self;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            _ => Err(Self::Invalid),
        }
    }
}
