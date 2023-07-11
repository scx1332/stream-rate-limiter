use futures::stream;
use futures::StreamExt;
use std::time::{Duration, Instant};
use stream_rate_limiter::{RateLimitOptions, StreamBehavior, StreamRateLimitExt};

extern crate stream_rate_limiter;

///Example using tokio interval, for most cases should be enough as good enough interval generator
#[tokio::main]
async fn main() {
    const GENERATE_ELEMENT_EVERY_SEC: f64 = 0.1;
    const DELAY_FOR: f64 = 3.0;

    let start = Instant::now();
    let _stream = stream::iter(0..100)
        .rate_limit(RateLimitOptions {
            interval: Some(Duration::from_secs_f64(GENERATE_ELEMENT_EVERY_SEC)),
            allowed_slippage_sec: Some(0.5),
            on_stream_delayed: |current_delay, total_delay| {
                let delay_for = current_delay;
                //println!("Stream is delayed {:.3}s !!", total_delay + delay_for);
                StreamBehavior::Continue
            },
        })
        .for_each(|el_no| async move {
            if el_no == 50 {
                tokio::time::sleep(Duration::from_secs_f64(DELAY_FOR)).await;
            }
            //note that after issue is solved, drift goes back to normal
            //println!(
            //    "El no {el_no}, Drift: {:.3}s",
            //    start.elapsed().as_secs_f64() - el_no as f64 * GENERATE_ELEMENT_EVERY_SEC
            //);
            println!(
                "{el_no} {:.3}",
                start.elapsed().as_secs_f64()
            );
        })
        .await;
}
