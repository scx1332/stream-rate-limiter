use crate::options::RateLimitOptions;
use crate::options::StreamBehavior;
use core::pin::Pin;
use futures_core::future::Future;
use futures_core::ready;
use futures_core::stream::{FusedStream, Stream};
use futures_core::task::{Context, Poll};
#[cfg(feature = "sink")]
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::time::Duration;
use tokio::time::Sleep;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct RateLimit<St>
    where St: Stream,
    {
        #[pin]
        stream: St,
        options: RateLimitOptions,
        #[pin]
        future: Option<Sleep>,
        item: Option<St::Item>,
        item_no: u64,
        first_el_time: std::time::Instant,
        stream_delay: f64,
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

impl<St> RateLimit<St>
where
    St: Stream,
{
    pub fn new(stream: St, opt: RateLimitOptions) -> Self {
        Self {
            stream,
            options: opt,
            item_no: 0,
            future: None,
            first_el_time: std::time::Instant::now(),
            item: None,
            stream_delay: 0.0,
        }
    }

    delegate_access_inner!(stream, St, ());
}

impl<St> FusedStream for RateLimit<St>
where
    St: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.future.is_none() && self.stream.is_terminated()
    }
}

impl<St> Stream for RateLimit<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                ready!(fut.poll(cx));
                this.future.set(None);
                break Some(this.item.take().unwrap());
            } else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                if *this.item_no == 0 {
                    *this.item_no += 1;
                    *this.first_el_time = std::time::Instant::now();
                    break Some(item);
                }
                if let Some(interval) = this.options.interval {
                    const MAX_SLIPPAGE_INTERVALS: f64 = 10.0;
                    const MAX_SLIPPAGE_CONST: f64 = 0.02;

                    let allowed_slippage_secs = this.options.allowed_slippage_sec.unwrap_or(
                        MAX_SLIPPAGE_INTERVALS * interval.as_secs_f64() + MAX_SLIPPAGE_CONST,
                    );

                    let target_time_point =
                        interval.as_secs_f64() * (*this.item_no) as f64 + *this.stream_delay;
                    *this.item_no += 1;

                    let elapsed = this.first_el_time.elapsed();
                    let delta = target_time_point - elapsed.as_secs_f64();
                    let wait_time_seconds = if delta > 0.0 { delta } else { 0.0 };
                    if delta < -(allowed_slippage_secs) {
                        let current_delay = -delta;
                        // stream is falling behind, add the permanent delay
                        if let Some(on_stream_delayed) = this.options.on_stream_delayed {
                            match on_stream_delayed(
                                current_delay,
                                *this.stream_delay + current_delay,
                            ) {
                                StreamBehavior::Continue => {}
                                StreamBehavior::Delay(delay) => {
                                    *this.stream_delay += delay;
                                }
                                StreamBehavior::Stop => break None,
                            }
                        } else {
                            *this.stream_delay += current_delay;
                        }
                    }
                    if wait_time_seconds > 0.001 {
                        this.future
                            .set(Some(tokio::time::sleep(Duration::from_secs_f64(
                                wait_time_seconds,
                            ))));
                        *this.item = Some(item);
                    } else {
                        break Some(item);
                    }
                } else {
                    break Some(item);
                }
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
    fn rate_limit(self, opt: RateLimitOptions) -> RateLimit<Self>
    where
        Self: Sized,
    {
        assert_stream::<Self::Item, _>(RateLimit::new(self, opt))
    }
}
pub(crate) fn assert_stream<T, S>(stream: S) -> S
where
    S: Stream<Item = T>,
{
    stream
}
