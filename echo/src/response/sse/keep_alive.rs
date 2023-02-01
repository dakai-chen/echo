use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::ready;
use tokio::time::{Instant, Sleep};

use super::Event;

#[derive(Debug, Clone)]
pub struct KeepAlive {
    text: Cow<'static, str>,
    interval: Duration,
}

impl KeepAlive {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.text = text.into();
        self
    }

    pub fn interval(mut self, time: Duration) -> Self {
        self.interval = time;
        self
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self {
            text: Cow::Borrowed(""),
            interval: Duration::from_secs(15),
        }
    }
}

pin_project_lite::pin_project! {
    pub(crate) struct KeepAliveStream {
        keep_alive: KeepAlive,
        #[pin]
        timer: Sleep,
    }
}

impl KeepAliveStream {
    pub(crate) fn new(keep_alive: KeepAlive) -> Self {
        Self {
            timer: tokio::time::sleep(keep_alive.interval),
            keep_alive,
        }
    }

    pub(crate) fn reset(self: Pin<&mut Self>) {
        let this = self.project();
        this.timer.reset(Instant::now() + this.keep_alive.interval);
    }

    pub(crate) fn poll_event(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Event> {
        ready!(self.as_mut().project().timer.poll(cx));

        self.as_mut().reset();

        Poll::Ready(Event::default().comment(self.keep_alive.text.clone()))
    }
}
