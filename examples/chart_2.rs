use futures::stream;
use futures::StreamExt;
use std::time::{Duration, Instant};
use stream_rate_limiter::{RateLimitOptions, StreamBehavior, StreamRateLimitExt};

extern crate stream_rate_limiter;

///Example using tokio interval, for most cases should be enough as good enough interval generator
#[tokio::main]
async fn main() {
    let start = Instant::now();
    let _stream = stream::iter(0..100)
        .rate_limit(RateLimitOptions {
            min_interval: Some(Duration::from_secs_f64(0.02)),
            interval: Some(Duration::from_secs_f64(0.1)),
            allowed_slippage_sec: Some(0.5),
            on_stream_delayed: Box::new(|_current_delay, _total_delay| StreamBehavior::Continue),
        })
        .for_each(|el_no| async move {
            if el_no == 40 {
                tokio::time::sleep(Duration::from_secs_f64(2.0)).await;
            }
            println!("{:.3}", start.elapsed().as_secs_f64());
        })
        .await;
}
