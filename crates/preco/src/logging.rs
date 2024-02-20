use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_tree::time::Uptime;
use tracing_tree::HierarchicalLayer;
pub fn setup_logging() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("preco=trace"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(
            HierarchicalLayer::default()
                .with_targets(true)
                .with_indent_lines(true)
                .with_timer(Uptime::default())
                .with_writer(std::io::stderr),
        )
        .init();
}
