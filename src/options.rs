use std::time::Duration;

pub enum StreamBehavior {
    Continue,
    Delay(f64),
    Stop,
}

#[derive(Clone, Default)]
pub struct RateLimitOptions<G>
where
    G: FnMut(f64, f64) -> StreamBehavior,
{
    ///targeted interval between items
    pub interval: Option<Duration>,

    ///none for default slippage (10 times interval + 0.02 sec)
    ///f64::max_value() for no slippage at all (stream always wants to catch up after delay)
    pub allowed_slippage_sec: Option<f64>,

    ///return true if you want
    pub on_stream_delayed: G,
}

impl<G> RateLimitOptions<G>
where
    G: FnMut(f64, f64) -> StreamBehavior,
{
    pub fn new(
        interval: Option<Duration>,
        allowed_slippage_sec: Option<f64>,
        on_stream_delayed: G,
    ) -> Self {
        Self {
            interval,
            allowed_slippage_sec,
            on_stream_delayed,
        }
    }
}
