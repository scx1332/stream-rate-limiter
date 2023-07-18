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
    pub struct RateLimit<'a, St>
    where St: Stream,
    {
        #[pin]
        stream: St,
        options: RateLimitOptions<'a>,
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

impl<'a, St> RateLimit<'a, St>
where
    St: Stream,
{
    pub fn new(stream: St, opt: RateLimitOptions<'a>) -> Self {
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

impl<'a, St> FusedStream for RateLimit<'a, St>
where
    St: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.future.is_none() && self.stream.is_terminated()
    }
}

impl<'a, St> Stream for RateLimit<'a, St>
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
                //item was set together with future, so we can safely return it
                break Some(this.item.take().unwrap());
            } else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                if *this.item_no == 0 {
                    *this.item_no += 1;
                    //start measuring time, when first element arrives
                    *this.first_el_time = std::time::Instant::now();
                    //return immediately on the first element
                    break Some(item);
                }
                //if no interval is set, use min_interval instead
                let interval = if this.options.interval.is_none() {
                    this.options.min_interval
                } else {
                    this.options.interval
                };

                if let Some(interval) = interval {
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
                    let mut wait_time_seconds = if delta > 0.0 { delta } else { 0.0 };
                    if delta < -(allowed_slippage_secs) {
                        let current_delay = -delta;
                        if let Some(on_stream_delayed) = this.options.on_stream_delayed.as_mut() {
                            match on_stream_delayed(
                                current_delay,
                                *this.stream_delay,
                            ) {
                                StreamBehavior::Continue => {}
                                StreamBehavior::Delay(delay) => {
                                    // stream is falling behind, add the permanent delay
                                    *this.stream_delay += delay;
                                }
                                StreamBehavior::Stop => break None,
                            }
                            //}
                        } else {
                            // stream is falling behind, add the permanent delay
                            *this.stream_delay += current_delay;
                        }
                    }
                    //if min interval is set, make sure we wait at least as long as min interval
                    if let Some(min_interval) = this.options.min_interval {
                        if min_interval.as_secs_f64() > wait_time_seconds {
                            wait_time_seconds = min_interval.as_secs_f64();
                        }
                    }
                    if wait_time_seconds > 0.001 || this.options.min_interval.is_some() {
                        // if min interval is provided wait always, even if it's zero

                        this.future
                            .set(Some(tokio::time::sleep(Duration::from_secs_f64(
                                wait_time_seconds,
                            ))));
                        //note that we are setting future here, so we can poll it,
                        //additionally we are setting item here, so we can return it when timeout is ready
                        *this.item = Some(item);
                    } else {
                        // if no min interval is provided, wait only if wait time bigger than 1ms
                        // otherwise return immediately to help catch up stream
                        break Some(item);
                    }
                } else {
                    //if no interval is set, return immediately (very low overhead)
                    break Some(item);
                }
            } else {
                //handle end of stream case
                break None;
            }
        })
    }

    //future is not changing stream size
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
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

/// import this trait to use `rate_limit` method on streams implementing `futures_core::stream::Stream`
pub trait StreamRateLimitExt: Stream {
    /// Rate limits the stream, behaviour depends on `RateLimitOptions` passed as argument
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
