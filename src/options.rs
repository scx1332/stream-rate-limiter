use std::time::Duration;

/// What to do with stream when it is delayed, returned in callback on_stream_delayed
pub struct StreamDelayedInfo {
    /// Number of element counted from the start of the stream
    pub element_no: u64,
    /// Current delay (in seconds)
    pub current_delay: f64,
    /// Total stream delay in seconds up to now (in seconds)
    pub total_delay: f64,
}

/// What to do with stream when it is delayed, returned in callback on_stream_delayed
pub enum StreamBehavior {
    ///Do not add delay to the stream, stream will try to catch up
    Continue,
    ///Add delay to the stream so it won't try to catch up
    Delay(f64),
    ///Stop stream (no longer producing elements and terminates without error)
    Stop,
}

/// Options to be used with rate_limit stream extension method
pub struct RateLimitOptions<'a> {
    pub(crate) interval: Option<Duration>,
    pub(crate) min_interval: Option<Duration>,
    pub(crate) allowed_slippage_sec: Option<f64>,
    pub(crate) on_stream_delayed:
        Option<Box<dyn FnMut(StreamDelayedInfo) -> StreamBehavior + Send + 'a>>,
}

impl<'a> RateLimitOptions<'a> {
    /// By default stream is not changed at all
    pub fn empty() -> Self {
        Self {
            interval: None,
            min_interval: None,
            allowed_slippage_sec: None,
            on_stream_delayed: None,
        }
    }

    ///Set targeted interval between items
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    ///Set targeted interval between items (in seconds)
    pub fn with_interval_sec(mut self, interval: f64) -> Self {
        self.interval = Some(Duration::from_secs_f64(interval));
        self
    }

    ///Set minimum interval between items
    pub fn with_min_interval(mut self, min_interval: Duration) -> Self {
        self.min_interval = Some(min_interval);
        self
    }

    ///Set minimum interval between items
    pub fn with_min_interval_sec(mut self, min_interval: f64) -> Self {
        self.min_interval = Some(Duration::from_secs_f64(min_interval));
        self
    }

    ///Set slippage - default slippage if not set (10 times interval + 0.02 sec) <br>
    ///if stream is currently delayed more than this then on_stream_delayed is called
    pub fn with_allowed_slippage_sec(mut self, allowed_slippage_sec: f64) -> Self {
        self.allowed_slippage_sec = Some(allowed_slippage_sec);
        self
    }

    ///Set callback called when stream is delayed more than allowed_slippage_sec <br/>
    ///return StreamBehavior::Delay to add permanent delay to stream <br/>
    ///return StreamBehavior::Stop to stop stream (terminate without error) <br/>
    ///return StreamBehavior::Continue to continue stream (trying catching up) <br/>
    ///First argument is current delay, second is permanent delay already in the stream
    pub fn on_stream_delayed(
        mut self,
        on_stream_delayed: impl FnMut(StreamDelayedInfo) -> StreamBehavior + 'a + std::marker::Send,
    ) -> Self {
        self.on_stream_delayed = Some(Box::new(on_stream_delayed));
        self
    }
}
