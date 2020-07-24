//! Future types
//!
use super::controller::Controller;
use super::instant_now;
use crate::sinks::util::retries2::RetryLogic;
use futures::ready;
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};
use tokio::sync::OwnedSemaphorePermit;

/// Future for the `AutoConcurrencyLimit` service.
#[pin_project]
#[derive(Debug)]
pub struct ResponseFuture<F, L> {
    #[pin]
    inner: F,
    // Keep this around so that it is dropped when the future completes
    _permit: OwnedSemaphorePermit,
    controller: Arc<Controller<L>>,
    start: Instant,
}

impl<F, L> ResponseFuture<F, L> {
    pub(super) fn new(
        inner: F,
        _permit: OwnedSemaphorePermit,
        controller: Arc<Controller<L>>,
    ) -> Self {
        Self {
            inner,
            _permit,
            controller,
            start: instant_now(),
        }
    }
}

impl<F, L, E> Future for ResponseFuture<F, L>
where
    F: Future<Output = Result<L::Response, E>>,
    L: RetryLogic,
    E: Into<crate::Error>,
{
    type Output = Result<L::Response, crate::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = self.project();
        let output = ready!(future.inner.poll(cx)).map_err(Into::into);
        future.controller.adjust_to_response(*future.start, &output);
        Poll::Ready(output)
    }
}