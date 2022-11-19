use std::str::FromStr;

#[derive(Debug)]
pub enum HttpMethod {
    Get,
}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &str {
        match *self {
            Self::Get => "GET",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            _ => Err("Invalid HTTP method"),
        }
    }
}
