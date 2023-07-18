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
    interval: Option<Duration>,

    ///minimum interval between items
    min_interval: Option<Duration>,

    ///none for default slippage (10 times interval + 0.02 sec)
    ///f64::max_value() for no slippage at all (stream always wants to catch up after delay)
    ///if stream is currently delayed more than this then on_stream_delayed is called
    allowed_slippage_sec: Option<f64>,

    ///return StreamBehavior::Delay to add permanent delay to stream
    ///return StreamBehavior::Stop to stop stream (terminate without error)
    ///return StreamBehavior::Continue to continue stream (trying catching up)
    ///First argument is current delay, second is permanent delay already in the stream
    on_stream_delayed: Box<dyn FnMut(f64, f64) -> StreamBehavior + 'a>,
}

pub fn on_stream_delayed_always_delayed(
    current_delay: f64,
    _permanent_delay: f64,
) -> StreamBehavior {
    StreamBehavior::Delay(current_delay)
}

impl<'a> Default for RateLimitOptions<'a> {
    fn default() -> Self {
        Self {
            interval: None,
            min_interval: None,
            allowed_slippage_sec: None,
            on_stream_delayed: Box::new(&on_stream_delayed_always_delayed),
        }
    }
}

impl<'a> RateLimitOptions<'a> {
    ///targeted interval between items
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn with_interval_sec(mut self, interval: f64) -> Self {
        self.interval = Some(Duration::from_secs_f64(interval));
        self
    }

    ///minimum interval between items
    pub fn with_min_interval(mut self, min_interval: Duration) -> Self {
        self.min_interval = Some(min_interval);
        self
    }

    pub fn with_min_interval_sec(mut self, min_interval: f64) -> Self {
        self.min_interval = Some(Duration::from_secs_f64(min_interval));
        self
    }

    ///none for default slippage (10 times interval + 0.02 sec)
    ///f64::max_value() for no slippage at all (stream always wants to catch up after delay)
    ///if stream is currently delayed more than this then on_stream_delayed is called
    pub fn with_allowed_slippage_sec(mut self, allowed_slippage_sec: f64) -> Self {
        self.allowed_slippage_sec = Some(allowed_slippage_sec);
        self
    }

    ///return StreamBehavior::Delay to add permanent delay to stream
    ///return StreamBehavior::Stop to stop stream (terminate without error)
    ///return StreamBehavior::Continue to continue stream (trying catching up)
    ///First argument is current delay, second is permanent delay already in the stream
    pub fn on_stream_delayed(
        mut self,
        on_stream_delayed: impl FnMut(f64, f64) -> StreamBehavior + 'a,
    ) -> Self {
        self.on_stream_delayed = Box::new(on_stream_delayed);
        self
    }

    pub fn interval(&self) -> Option<Duration> {
        self.interval
    }

    pub fn min_interval(&self) -> Option<Duration> {
        self.min_interval
    }

    pub fn allowed_slippage_sec(&self) -> Option<f64> {
        self.allowed_slippage_sec
    }

    pub fn get_on_stream_delayed(&mut self) -> &mut dyn FnMut(f64, f64) -> StreamBehavior {
        self.on_stream_delayed.as_mut()
    }
}
