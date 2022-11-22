use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};

pub enum HttpMethod {
    Get,
    Post,
}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &str {
        match *self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err("Invalid HTTP method"),
        }
    }
}

pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpRequest {
    pub fn new(
        method: HttpMethod,
        path: String,
        version: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            version,
            headers,
            body,
        }
    }

    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &String {
        &self.body
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub async fn from_stream_reader(
        reader: Arc<Mutex<BufReader<TcpStream>>>,
    ) -> Result<Self, &'static str> {
        let mut start = String::new();
        let (method, path, version) = Self::parse_start(Arc::clone(&reader), &mut start).await?;

        let mut line = String::new();
        let headers = Self::parse_headers(Arc::clone(&reader), &mut line).await?;

        let body_len = headers
            .get("Content-Length")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .unwrap();
        let body = vec![0u8; body_len];
        let body = Self::parse_body(Arc::clone(&reader), body).await?;

        let method = HttpMethod::from_str(method)?;
        Ok(Self {
            method,
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body: String::from_utf8_lossy(body.as_slice()).to_string(),
        })
    }

    async fn parse_start<'start>(
        reader: Arc<Mutex<BufReader<TcpStream>>>,
        start: &'start mut String,
    ) -> Result<(&'start str, &'start str, &'start str), &'static str> {
        let mut reader = reader.lock().await;

        if let Err(_) = reader.read_line(start).await {
            return Err("Failed to read startline");
        }
        let start_parts = start.split_whitespace().collect::<Vec<&str>>();

        if start_parts.len() != 3 {
            return Err("Invalid HTTP request".into());
        }

        Ok((start_parts[0], start_parts[1], start_parts[2]))
    }

    async fn parse_headers(
        reader: Arc<Mutex<BufReader<TcpStream>>>,
        line: &mut String,
    ) -> Result<HashMap<String, String>, &'static str> {
        let mut reader = reader.lock().await;
        let mut headers = HashMap::<String, String>::new();

        loop {
            if let Err(_) = reader.read_line(line).await {
                return Err("Failed to read headers");
            }
            if *line == "\r\n" {
                break;
            }

            *line = line.trim_end().to_string();
            let parts = line.split_once(": ").unwrap();
            let (header, val) = (parts.0.to_string(), parts.1.to_string());

            headers.insert(header, val);

            *line = String::new();
        }

        Ok(headers)
    }

    async fn parse_body(
        reader: Arc<Mutex<BufReader<TcpStream>>>,
        mut body: Vec<u8>,
    ) -> Result<Vec<u8>, &'static str> {
        let mut reader = reader.lock().await;

        if let Err(_) = reader.read_exact(&mut body).await {
            return Err("Failed to read body");
        }
        Ok(body)
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = String::new();
        let start = format!(
            "{} {} {}\r\n",
            self.method.as_ref(),
            self.path,
            self.version
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

pub trait IntoResponse {
    fn into_response(self) -> HttpResponse;
}

impl<A> IntoResponse for A
where
    A: AsRef<str>,
{
    fn into_response(self) -> HttpResponse {
        let mut res = HttpResponse::new(HttpStatus::OK);
        res.set_header("Content-Type", "text/plain");
        res.set_body(self);
        res
    }
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> HttpResponse {
        self
    }
}
