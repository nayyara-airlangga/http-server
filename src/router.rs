use std::{collections::HashMap, future::Future, pin::Pin};

use crate::message::{HttpMethod, HttpRequest, HttpResponse};

pub trait Handler {
    type Fut: Future<Output = HttpResponse>;

    fn call(self, req: HttpRequest) -> Self::Fut;
}

impl<F, Fut> Handler for F
where
    F: FnOnce(HttpRequest) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = HttpResponse> + Send,
{
    type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

    fn call(self, req: HttpRequest) -> Self::Fut {
        Box::pin(async move { self(req).await })
    }
}

pub struct Route {
    methods: HashMap<
        &'static str,
        Pin<Box<dyn Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>>>,
    >,
}

impl Route {
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    pub fn get<H>(mut self, handler: H) -> Self
    where
        H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
    {
        self.methods
            .insert(HttpMethod::Get.as_ref(), Box::pin(handler));
        self
    }

    pub fn post<H>(mut self, handler: H) -> Self
    where
        H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
    {
        self.methods
            .insert(HttpMethod::Post.as_ref(), Box::pin(handler));
        self
    }
}

pub fn get<H>(handler: H) -> Route
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
{
    let mut route = Route::new();
    route
        .methods
        .insert(HttpMethod::Get.as_ref(), Box::pin(handler));
    route
}

pub fn post<H>(handler: H) -> Route
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
{
    let mut route = Route::new();
    route
        .methods
        .insert(HttpMethod::Post.as_ref(), Box::pin(handler));
    route
}

pub struct Router {
    routes: HashMap<&'static str, Route>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn route(mut self, path: &'static str, route: Route) -> Self {
        self.routes.insert(path, route);
        self
    }
}
