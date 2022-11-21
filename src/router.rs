use std::{collections::HashMap, future::Future, pin::Pin};

use crate::message::{HttpMethod, HttpRequest, HttpResponse};

pub(crate) trait AsyncFnBox {
    type Fut: Future<Output = HttpResponse> + Send;

    fn call_box(&self, req: HttpRequest) -> Self::Fut;
}

impl<H> AsyncFnBox for H
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + ?Sized,
{
    type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

    fn call_box(&self, req: HttpRequest) -> Self::Fut {
        Box::pin(self.call(req))
    }
}

pub trait Handler: Send + Sync {
    type Fut: Future<Output = HttpResponse>;

    fn call(&self, req: HttpRequest) -> Self::Fut;
}

impl<F, Fut> Handler for F
where
    F: FnOnce(HttpRequest) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = HttpResponse> + Send,
{
    type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

    fn call(&self, req: HttpRequest) -> Self::Fut {
        Box::pin(self.call_box(req))
    }
}

pub struct Route {
    methods: HashMap<
        &'static str,
        Box<dyn Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>>,
    >,
}

impl Route {
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    pub fn methods(
        &self,
    ) -> &HashMap<
        &'static str,
        Box<dyn Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>>,
    > {
        &self.methods
    }

    pub fn get<H>(mut self, handler: H) -> Self
    where
        H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
    {
        self.methods
            .insert(HttpMethod::Get.as_ref(), Box::new(handler));
        self
    }

    pub fn post<H>(mut self, handler: H) -> Self
    where
        H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
    {
        self.methods
            .insert(HttpMethod::Post.as_ref(), Box::new(handler));
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
        .insert(HttpMethod::Get.as_ref(), Box::new(handler));
    route
}

pub fn post<H>(handler: H) -> Route
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + 'static,
{
    let mut route = Route::new();
    route
        .methods
        .insert(HttpMethod::Post.as_ref(), Box::new(handler));
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
