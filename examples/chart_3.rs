use futures::stream;
use futures::StreamExt;
use plotly::color::Rgb;
use plotly::common::{Font, Line, Marker, Mode, Title};
use plotly::layout::{Axis, Margin, TicksDirection};
use plotly::{Layout, Plot, Scatter};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use std::time::{Duration, Instant};
use stream_rate_limiter::{RateLimitOptions, StreamBehavior, StreamRateLimitExt};

extern crate stream_rate_limiter;

///Example using tokio interval, for most cases should be enough as good enough interval generator
#[tokio::main]
async fn main() {
    let start = Instant::now();
    let plot_x = Rc::new(RefCell::new(vec![]));
    let plot_y = Rc::new(RefCell::new(vec![]));

    let _plot_x = plot_x.clone();
    let _plot_y = plot_y.clone();
    let _stream = stream::iter(0..101)
        .rate_limit(
            RateLimitOptions::empty()
                .with_min_interval_sec(0.02)
                .with_interval_sec(0.1)
                .with_allowed_slippage_sec(0.5)
                .on_stream_delayed(|_sdi| StreamBehavior::Stop),
        )
        .for_each(move |el_no| {
            let plot_x = _plot_x.clone();
            let plot_y = _plot_y.clone();
            async move {
                if el_no == 40 {
                    tokio::time::sleep(Duration::from_secs_f64(2.0)).await;
                }
                plot_x.borrow_mut().push(el_no);
                plot_y.borrow_mut().push(start.elapsed().as_secs_f64());
                //println!("{:.3}", start.elapsed().as_secs_f64());
            }
        })
        .await;

    let mut plot = Plot::new();
    let title_font = Font::new().size(25);
    let axis_font = Font::new().size(25);
    let layout = Layout::new()
        .show_legend(false)
        .margin(Margin::new().top(25).bottom(80).left(80).right(30))
        .x_axis(
            Axis::new()
                .title(Title::new("Element no.").font(title_font.clone()))
                .tick_font(axis_font.clone())
                .ticks(TicksDirection::Outside)
                .show_spikes(true)
                .range(vec![0.0, 102.0]),
        )
        .y_axis(
            Axis::new()
                .title(Title::new("Time of event").font(title_font))
                .tick_font(axis_font)
                .ticks(TicksDirection::Outside)
                .show_spikes(true)
                .range(vec![0.0, 12.1]),
        );
    let plot_x = plot_x.borrow().to_vec();
    let plot_y = plot_y.borrow().to_vec();
    let trace = Scatter::new(plot_x.clone(), plot_y)
        .mode(Mode::Markers)
        .marker(Marker::new().size(7).color(Rgb::new(0, 0, 255)))
        .text("Elements");

    let trace2 = {
        let plot_y = plot_x.iter().map(|el| *el as f64 * 0.1).collect();
        Scatter::new(plot_x, plot_y)
            .mode(Mode::Lines)
            .line(Line::new().color(Rgb::new(155, 155, 155)))
            .text("Ideal no delay")
    };

    plot.set_layout(layout);
    plot.add_trace(trace2);
    plot.add_trace(trace);

    let mut file = File::create("chart_3.html").unwrap();
    file.write_all(
        plot.to_html()
            .replace(
                r#"style="height:100%; width:100%;""#,
                r#"style="height:100vh; max-height: 800px; width:100%; max-width: 1280px; ""#,
            )
            .as_bytes(),
    )
    .expect("failed to write html output");
}
