use core::fmt;
use core::pin::Pin;
use futures::{stream, StreamExt};
use futures_core::future::Future;
use futures_core::ready;
use futures_core::stream::{FusedStream, Stream};
use futures_core::task::{Context, Poll};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::time::Duration;


pin_project! {
    /// Stream for the [`then`](super::StreamExt::then) method.
    #[must_use = "streams do nothing unless polled"]
    pub struct RateLimit<St, Fut, F> {
        #[pin]
        stream: St,
        #[pin]
        future: Option<Fut>,
        options: RateLimitOptions,
        f: F,
    }
}

impl<St, Fut, F> fmt::Debug for RateLimit<St, Fut, F>
where
    St: fmt::Debug,
    Fut: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RateLimit")
            .field("stream", &self.stream)
            .field("future", &self.future)
            .finish()
    }
}

macro_rules! delegate_access_inner {
    ($field:ident, $inner:ty, ($($ind:tt)*)) => {
        /// Acquires a reference to the underlying sink or stream that this combinator is
        /// pulling from.
        pub fn get_ref(&self) -> &$inner {
            (&self.$field) $($ind get_ref())*
        }

        /// Acquires a mutable reference to the underlying sink or stream that this
        /// combinator is pulling from.
        ///
        /// Note that care must be taken to avoid tampering with the state of the
        /// sink or stream which may otherwise confuse this combinator.
        pub fn get_mut(&mut self) -> &mut $inner {
            (&mut self.$field) $($ind get_mut())*
        }

        /// Acquires a pinned mutable reference to the underlying sink or stream that this
        /// combinator is pulling from.
        ///
        /// Note that care must be taken to avoid tampering with the state of the
        /// sink or stream which may otherwise confuse this combinator.
        pub fn get_pin_mut(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut $inner> {
            self.project().$field $($ind get_pin_mut())*
        }

        /// Consumes this combinator, returning the underlying sink or stream.
        ///
        /// Note that this may discard intermediate state of this combinator, so
        /// care should be taken to avoid losing resources when this is called.
        pub fn into_inner(self) -> $inner {
            self.$field $($ind into_inner())*
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitOptions {
    pub interval: Duration,
}

impl<St, Fut, F> RateLimit<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
{
    pub fn new(stream: St, opt: RateLimitOptions, f: F) -> Self {
        Self {
            stream,
            future: None,
            f,
            options: opt,
        }
    }

    delegate_access_inner!(stream, St, ());
}

impl<St, Fut, F> FusedStream for RateLimit<St, Fut, F>
where
    St: FusedStream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future,
{
    fn is_terminated(&self) -> bool {
        self.future.is_none() && self.stream.is_terminated()
    }
}

impl<St, Fut, F> Stream for RateLimit<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future,
{
    type Item = Fut::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.future.set(None);
                break Some(item);
            } else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                this.future.set(Some((this.f)(item)));
            } else {
                break None;
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let future_len = usize::from(self.future.is_some());
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(future_len);
        let upper = match upper {
            Some(x) => x.checked_add(future_len),
            None => None,
        };
        (lower, upper)
    }
}

// Forwarding impl of Sink from the underlying stream
#[cfg(feature = "sink")]
impl<S, Fut, F, Item> Sink<Item> for RateLimit<S, Fut, F>
where
    S: Sink<Item>,
{
    type Error = S::Error;

    delegate_sink!(stream, Item);
}

impl<T: ?Sized> StreamRateLimitExt for T where T: Stream {}

pub trait StreamRateLimitExt: Stream {
    fn rate_limit<Fut, F>(self, opt: RateLimitOptions, f: F) -> RateLimit<Self, Fut, F>
    where
        F: FnMut(Self::Item) -> Fut,
        Fut: Future,
        Self: Sized,
    {
        assert_stream::<Fut::Output, _>(RateLimit::new(self, opt, f))
    }
    /*fn rate_limit2<Fut, F>(self, opt: RateLimitOptions) -> RateLimit<Self, Fut, F>
        where
            F: FnMut(Self::Item) -> Fut,
            Fut: Future,
            Self: Sized,
    {
        let f = |x: Self::Item| {
            async move { x };
        };
        self.rate_limit(opt, f)
    }*/
}
pub(crate) fn assert_stream<T, S>(stream: S) -> S
where
    S: Stream<Item = T>,
{
    stream
}
