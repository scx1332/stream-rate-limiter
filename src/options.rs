use std::time::Duration;

pub enum StreamBehavior {
    ///Do not add delay to the stream, stream will try to catch up
    Continue,
    ///Add delay to the stream so it won't try to catch up
    Delay(f64),
    ///Stop stream (no longer producing elements and terminates without error)
    Stop,
}

pub struct RateLimitOptions<'a> {
    ///targeted interval between items
    pub interval: Option<Duration>,

    ///minimum interval between items
    pub min_interval: Option<Duration>,

    ///none for default slippage (10 times interval + 0.02 sec)
    ///f64::max_value() for no slippage at all (stream always wants to catch up after delay)
    ///if stream is currently delayed more than this then on_stream_delayed is called
    pub allowed_slippage_sec: Option<f64>,

    ///return StreamBehavior::Delay to add permanent delay to stream
    ///return StreamBehavior::Stop to stop stream (terminate without error)
    ///return StreamBehavior::Continue to continue stream (trying catching up)
    ///First argument is current delay, second is permanent delay already in the stream
    pub on_stream_delayed: &'a mut dyn FnMut(f64, f64) -> StreamBehavior,
}

impl<'a> RateLimitOptions<'a> {
    pub fn new(
        interval: Option<Duration>,
        min_interval: Option<Duration>,
        allowed_slippage_sec: Option<f64>,
        on_stream_delayed: &'a mut dyn FnMut(f64, f64) -> StreamBehavior,
    ) -> Self {
        Self {
            interval,
            min_interval,
            allowed_slippage_sec,
            on_stream_delayed,
        }
    }
}
