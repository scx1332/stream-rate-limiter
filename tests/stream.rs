#[cfg(test)]
mod tests {
    use futures::stream;
    use futures::stream::StreamExt;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;
    use stream_rate_limiter::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn it_works() {
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(None, None, None, &mut |_, _| {
                StreamBehavior::Continue
            }))
            .count()
            .await;
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn it_works2() {
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                Some(Duration::from_secs_f64(0.01)),
                None,
                None,
                &mut |_, _| StreamBehavior::Continue,
            ))
            .count()
            .await;
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn it_works3() {
        let total_delay = Rc::new(RefCell::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                Some(Duration::from_secs_f64(0.01)),
                None,
                None,
                |delta, stream_delay| {
                    total_delay.replace_with(|_| stream_delay + delta);
                    println!("Stream is delayed {:.3}s !!", stream_delay + delta);
                    StreamBehavior::Delay(delta)
                },
            ))
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        println!("Total delay: {}", *total_delay.borrow());
        assert_eq!(count, 10);
        assert!(*total_delay.borrow() > 0.5);
        assert!(*total_delay.borrow() < 1.0);
    }

    #[tokio::test]
    async fn it_works4() {
        let total_delay = Rc::new(RefCell::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                Some(Duration::from_secs_f64(0.01)),
                None,
                None,
                &mut |_delta, stream_delay| {
                    total_delay.replace_with(|_| stream_delay);
                    println!("Stream is delayed {stream_delay:.3}s !!");
                    StreamBehavior::Continue
                },
            ))
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        println!("Total delay: {}", *total_delay.borrow());
        assert_eq!(count, 10);
        assert_eq!(*total_delay.borrow(), 0.0);
    }

    #[tokio::test]
    async fn it_works5() {
        let total_delay = Rc::new(RefCell::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                Some(Duration::from_secs_f64(0.01)),
                None,
                Some(10.0),
                &mut |delta, stream_delay| {
                    total_delay.replace_with(|_| stream_delay + delta);
                    println!("Stream is delayed {:.3}s !!", stream_delay + delta);
                    StreamBehavior::Delay(delta)
                },
            ))
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        println!("Total delay: {}", *total_delay.borrow());
        assert_eq!(count, 10);
        assert_eq!(*total_delay.borrow(), 0.0);
    }

    #[tokio::test]
    async fn it_works6() {
        let total_delay = Rc::new(RefCell::new(0.0));
        let instant = Instant::now();
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                None,
                Some(Duration::from_secs_f64(0.1)),
                None,
                &mut |delta, stream_delay| {
                    total_delay.replace_with(|_| stream_delay + delta);
                    println!("Stream is delayed {:.3}s !!", stream_delay + delta);
                    StreamBehavior::Delay(delta)
                },
            ))
            .count()
            .await;
        println!("Total delay: {}", *total_delay.borrow());
        assert_eq!(count, 10);
        assert_eq!(*total_delay.borrow(), 0.0);
        assert!(instant.elapsed().as_secs_f64() > 0.9);
    }
}
