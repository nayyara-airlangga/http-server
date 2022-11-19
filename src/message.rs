use std::{collections::HashMap, fmt::Display, str::FromStr};

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

pub enum HttpStatus {
    OK,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
}

impl HttpStatus {
    pub fn code(&self) -> u16 {
        match *self {
            Self::OK => 200,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::InternalServerError => 500,
        }
    }
}

impl AsRef<str> for HttpStatus {
    fn as_ref(&self) -> &str {
        match *self {
            Self::OK => "OK",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::InternalServerError => "Internal Server Error",
        }
    }
}

pub struct HttpResponse {
    pub status: HttpStatus,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        let mut res = HttpResponse {
            status,
            headers: HashMap::new(),
            body: String::new(),
        };

        res.set_header("Content-Length", "0");

        res
    }

    pub fn set_header(&mut self, header: impl AsRef<str>, val: impl AsRef<str>) -> Option<String> {
        self.headers
            .insert(header.as_ref().to_string(), val.as_ref().to_string())
    }

    pub fn remove_header(&mut self, header: impl AsRef<str>) -> Option<String> {
        self.headers.remove(header.as_ref())
    }

    pub fn set_body(&mut self, body: impl AsRef<str>) {
        self.set_header("Content-Length", body.as_ref().as_bytes().len().to_string());
        self.body = body.as_ref().to_string();
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = String::new();
        let start = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status.code(),
            self.status.as_ref()
        );
        message.push_str(&start);

        for (k, v) in self.headers.iter() {
            let header_str = format!("{}: {}\r\n", k, v);
            message.push_str(&header_str)
        }
        message.push_str("\r\n");
        message.push_str(&self.body);

        write!(f, "{message}")
    }
}
