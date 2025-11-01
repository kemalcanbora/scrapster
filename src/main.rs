use std::time::Duration;
use scrapster::MetricsCollector;

#[tokio::main]
async fn main() {
    let collector = MetricsCollector::new(Duration::from_secs(1));
    let mut rx = collector.start().await;

    while let Ok(metrics) = rx.recv().await {
        println!("{:?}", metrics);
    }
}
