#[cfg(test)]
mod tests {
    use futures::stream;
    use futures::stream::StreamExt;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use stream_rate_limiter::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn it_works() {
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::empty())
            .count()
            .await;
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn it_works2() {
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::empty().with_interval_sec(0.01))
            .count()
            .await;
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn it_works3() {
        let total_delay = Arc::new(Mutex::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(
                RateLimitOptions::empty()
                    .with_interval_sec(0.01)
                    .on_stream_delayed(|sdi| {
                        *total_delay.lock().unwrap() = sdi.total_delay + sdi.current_delay;
                        println!(
                            "Stream is delayed {:.3}s on el {}!!",
                            sdi.total_delay + sdi.current_delay,
                            sdi.element_no
                        );
                        StreamBehavior::Delay(sdi.current_delay)
                    }),
            )
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        println!("Total delay: {}", *total_delay.lock().unwrap());
        assert_eq!(count, 10);
        assert!(*total_delay.lock().unwrap() > 0.5);
        assert!(*total_delay.lock().unwrap() < 1.0);
    }

    #[tokio::test]
    async fn it_works4() {
        let total_delay = Arc::new(Mutex::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(
                RateLimitOptions::empty()
                    .with_interval_sec(0.01)
                    .on_stream_delayed(|sdi| {
                        *total_delay.lock().unwrap() = sdi.total_delay;
                        println!("Stream is delayed {:.3}s !!", sdi.total_delay);
                        StreamBehavior::Continue
                    }),
            )
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        println!("Total delay: {}", *total_delay.lock().unwrap());
        assert_eq!(count, 10);
        assert_eq!(*total_delay.lock().unwrap(), 0.0);
    }

    #[tokio::test]
    async fn it_works5() {
        let total_delay = Arc::new(Mutex::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(
                RateLimitOptions::empty()
                    .with_interval_sec(0.01)
                    .with_allowed_slippage_sec(10.0)
                    .on_stream_delayed(|sdi| {
                        *total_delay.lock().unwrap() = sdi.current_delay + sdi.total_delay;
                        println!(
                            "Stream is delayed {:.3}s !!",
                            sdi.current_delay + sdi.total_delay
                        );
                        StreamBehavior::Delay(sdi.current_delay)
                    }),
            )
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        println!("Total delay: {}", *total_delay.lock().unwrap());
        assert_eq!(count, 10);
        assert_eq!(*total_delay.lock().unwrap(), 0.0);
    }

    #[tokio::test]
    async fn it_works6() {
        let total_delay = Arc::new(Mutex::new(0.0));
        let instant = Instant::now();
        let count = stream::iter(0..10)
            .rate_limit(
                RateLimitOptions::empty()
                    .with_min_interval_sec(0.1)
                    .on_stream_delayed(|sdi| {
                        *total_delay.lock().unwrap() = sdi.current_delay + sdi.total_delay;
                        println!(
                            "Stream is delayed {:.3}s !!",
                            sdi.current_delay + sdi.total_delay
                        );
                        StreamBehavior::Delay(sdi.current_delay)
                    }),
            )
            .count()
            .await;
        println!("Total delay: {}", *total_delay.lock().unwrap());
        assert_eq!(count, 10);
        assert_eq!(*total_delay.lock().unwrap(), 0.0);
        assert!(instant.elapsed().as_secs_f64() > 0.9);
    }
}
