use std::{future::Future, pin::Pin};

use crate::message::{HttpRequest, HttpResponse, IntoResponse};

pub trait Handler: Send + Sync + Clone + Sized + 'static {
    type Fut: Future<Output = HttpResponse>;

    fn call(self, req: HttpRequest) -> Self::Fut;
}

pub trait Service {
    type Future: Future<Output = HttpResponse> + Send;

    fn call(&self, req: HttpRequest) -> Self::Future;
}

pub struct HandlerService<H> {
    handler: H,
}

impl<H> HandlerService<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }
}

impl<H> Clone for HandlerService<H>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }
}

impl<H> Service for HandlerService<H>
where
    H: Handler<Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>> + Clone + Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;

    fn call(&self, req: HttpRequest) -> Self::Future {
        let handler = self.handler.clone();
        Handler::call(handler, req)
    }
}

pub struct BoxCloneService(
    Box<dyn CloneService<Future = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>>,
);

impl BoxCloneService {
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Future = Pin<Box<dyn Future<Output = HttpResponse> + Send>>>
            + Send
            + Sync
            + Clone
            + 'static,
    {
        Self(Box::new(service))
    }
}

impl Service for BoxCloneService {
    type Future = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;
    fn call(&self, req: HttpRequest) -> Self::Future {
        self.0.call(req)
    }
}

impl Clone for BoxCloneService {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

pub trait CloneService: Service + Sync {
    fn clone_box(&self) -> Box<dyn CloneService<Future = Self::Future> + Send>;
}

impl<S> CloneService for S
where
    S: Service + Send + Clone + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn CloneService<Future = S::Future> + Send> {
        Box::new(self.clone())
    }
}

impl<F, Fut, Res> Handler for F
where
    F: Fn(HttpRequest) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
{
    type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;

    fn call(self, req: HttpRequest) -> Self::Fut {
        Box::pin(async move { self(req).await.into_response() })
    }
}
