# stream-rate-limiter

Stream combinator

.rate_limiter(...)

Provides way to limit stream element rate with constant intervals. 
It adds some level of customization in case of stream delays 
inaccessible in "standard" solution (tokio-timer)

Before you use this library, please consider using tokio-timer crate
(you can look at tokio_interval example)

If you need to limit stream rate with constant intervals, with additional
customisation (increasing permanent stream delay in case of stream delays, which
prevents flooding stream after huge hickups.

For example you want to produce a stream of elements with constant rate of 1 element per second.
You can use this library to set rate limit to 1 element and for example 1 second of accepted delay.
Thus after your stream is stuck for like couple of seconds your event won't be flooding but they go back to rate at one 
per second fast. You can also react and close the stream if delay is too big.


## Case study

x axis is element number, y axis is time, el number 50 is simulated to stuck for 3 seconds)

When you want to delay stream after hiccup 
```rust
stream::iter(0..100)
    .rate_limit(RateLimitOptions {
        min_interval: Some(Duration::from_secs_f64(0.02)),
        interval: Some(Duration::from_secs_f64(0.1)),
        allowed_slippage_sec: Some(0.5),
        on_stream_delayed: |current_delay, _total_delay| {
            StreamBehavior::Delay(current_delay)
        },
    })
```
![alt text](https://github.com/scx1332/stream-rate-limiter/blob/main/docs/chart_1.png?raw=true)

When you want to allow stream to catchup after hiccup (continue option):
```rust
stream::iter(0..100)
    .rate_limit(RateLimitOptions {
        min_interval: Some(Duration::from_secs_f64(0.02)),
        interval: Some(Duration::from_secs_f64(0.1)),
        allowed_slippage_sec: Some(0.5),
        on_stream_delayed: |_current_delay, _total_delay| StreamBehavior::Continue,
        })
```
![alt text](https://github.com/scx1332/stream-rate-limiter/blob/main/docs/chart_2.png?raw=true)


When you want to stop stream after hiccup (stop option):
```rust
stream::iter(0..100)
    .rate_limit(RateLimitOptions {
        min_interval: Some(Duration::from_secs_f64(0.02)),
        interval: Some(Duration::from_secs_f64(0.1)),
        allowed_slippage_sec: Some(0.5),
        on_stream_delayed: |_current_delay, _total_delay| StreamBehavior::Stop,
        })
```
![alt text](https://github.com/scx1332/stream-rate-limiter/blob/main/docs/chart_3.png?raw=true)


