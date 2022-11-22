use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{
    message::{HttpMethod, HttpResponse},
    service::{BoxCloneService, Handler, HandlerService},
};

pub struct Route {
    methods: HashMap<&'static str, BoxCloneService>,
}

impl Route {
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    pub fn methods(&self) -> &HashMap<&'static str, BoxCloneService> {
        &self.methods
    }

    pub fn get<H>(mut self, handler: H) -> Self
    where
        H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>
            + Send
            + Clone
            + 'static,
    {
        let handler = HandlerService::new(handler);
        self.methods
            .insert(HttpMethod::Get.as_ref(), BoxCloneService::new(handler));
        self
    }

    pub fn post<H>(mut self, handler: H) -> Self
    where
        H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>
            + Send
            + Clone
            + 'static,
    {
        let handler = HandlerService::new(handler);
        self.methods
            .insert(HttpMethod::Post.as_ref(), BoxCloneService::new(handler));
        self
    }
}

pub fn get<H>(handler: H) -> Route
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + Send + Clone + 'static,
{
    let mut route = Route::new();
    let handler = HandlerService::new(handler);
    route
        .methods
        .insert(HttpMethod::Get.as_ref(), BoxCloneService::new(handler));
    route
}

pub fn post<H>(handler: H) -> Route
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + Send + Clone + 'static,
{
    let mut route = Route::new();
    let handler = HandlerService::new(handler);
    route
        .methods
        .insert(HttpMethod::Post.as_ref(), BoxCloneService::new(handler));
    route
}

pub struct Router {
    routes: HashMap<&'static str, Route>,
}

unsafe impl Send for Router {}

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

    pub fn routes(&self) -> &HashMap<&'static str, Route> {
        &self.routes
    }
}
