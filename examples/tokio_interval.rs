use futures::{StreamExt};
use std::time::Duration;
use tokio::time::{interval, Instant};
use tokio_stream::wrappers::IntervalStream;

///Example using tokio interval, for most cases should be enough as good enough interval generator
#[tokio::main]
async fn main() {
    const GENERATE_ELEMENT_EVERY_SEC: f64 = 0.04;
    const DELAY_FOR: f64 = 0.08;

    let start = Instant::now();
    let stream = IntervalStream::new(interval(Duration::from_secs_f64(
        GENERATE_ELEMENT_EVERY_SEC,
    )));
    stream
        .enumerate()
        .for_each(|(el_no, _el)| async move {
            if el_no > 50 && el_no < 100 {
                tokio::time::sleep(Duration::from_secs_f64(DELAY_FOR)).await;
            }
            //note that after issue is solved, drift goes back to normal
            println!(
                "El no {el_no}, Drift: {:.3}s",
                start.elapsed().as_secs_f64() - el_no as f64 * GENERATE_ELEMENT_EVERY_SEC
            );
        })
        .await;
}
