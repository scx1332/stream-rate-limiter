#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::*;
    use futures::executor::block_on;
    use futures::stream;
    use futures::stream::StreamExt;
    use std::time::Duration;
    use stream_rate_limiter::*;

    #[tokio::test]
    async fn it_works() {
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                None,
                None,
                None,
            ))
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
            ))
            .count()
            .await;
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn it_works3() {
/*
        let total_delay = Rc::new(RefCell::new(0.0));
        let count = stream::iter(0..10)
            .rate_limit(RateLimitOptions::new(
                Some(Duration::from_secs_f64(0.01)),
                None,
                Some(|delta, total_delta| {
                    total_delay.replace_with(|&mut x| total_delta);
                    println!("Stream is delayed !!");
                    StreamBehavior::Delay(delta)
                }),
            ))
            .then(|_| async { tokio::time::sleep(Duration::from_secs_f64(0.1)).await })
            .count()
            .await;
        assert_eq!(count, 10);*/
    }

}
