mod compat;
mod graceful_shutdown;
use graceful_shutdown::GracefulShutdown;

use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;

use echo_core::response::IntoResponse;
use echo_core::service::{Service, ServiceExt};
use echo_core::{BoxError, Request};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use tokio::net::TcpListener;

use crate::server::compat::{EchoToHyper, HyperToEcho};

#[derive(Debug, Clone)]
struct Options {
    addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct Server {
    options: Options,
    http1: http1::Builder,
}

impl Server {
    pub fn bind(addr: SocketAddr) -> Self {
        Self {
            options: Options { addr },
            http1: http1::Builder::new(),
        }
    }

    pub fn cfg_http1<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut http1::Builder),
    {
        f(&mut self.http1);
        self
    }

    pub async fn serve<S>(self, service: S) -> Result<(), BoxError>
    where
        S: Service<Request, Error = Infallible> + Send + Sync + 'static,
        S::Response: IntoResponse,
        for<'f> S::Future<'f>: Send,
    {
        self.serve_with_graceful_shutdown(service, std::future::pending())
            .await
    }

    pub async fn serve_with_graceful_shutdown<S, G>(
        self,
        service: S,
        signal: G,
    ) -> Result<(), BoxError>
    where
        S: Service<Request, Error = Infallible> + Send + Sync + 'static,
        S::Response: IntoResponse,
        for<'f> S::Future<'f>: Send,
        G: Future<Output = Option<Duration>> + Send + 'static,
    {
        tokio::pin!(signal);

        let service = service.boxed_arc();
        let service = hyper::service::service_fn(move |req| {
            let service = service.clone();
            async move {
                let service = service
                    .map_request(|request: Request<Incoming>| request.map(HyperToEcho::to))
                    .map_ok(|response: S::Response| response.into_response().map(EchoToHyper::to));
                service.call(req).await
            }
        });

        let graceful = GracefulShutdown::new();
        let listener = TcpListener::bind(self.options.addr).await?;

        let timeout = loop {
            tokio::select! {
                timeout = signal.as_mut() => {
                    break timeout;
                }
                conn = listener.accept() => {
                    let (conn, _) = conn?;

                    let conn = self.http1.serve_connection(conn, service.clone()).with_upgrades();
                    let conn = graceful.watch(conn);

                    tokio::spawn(conn);
                }
            }
        };

        graceful.shutdown(timeout).await;

        Ok(())
    }
}
