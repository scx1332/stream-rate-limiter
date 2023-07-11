use futures::stream;
use futures::stream::StreamExt;
use std::time::{Duration, Instant};

mod options;
mod streamext;

pub use options::StreamBehavior;
pub use options::RateLimitOptions;
pub use streamext::StreamRateLimitExt;




async fn sample() {

}
