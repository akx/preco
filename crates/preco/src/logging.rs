use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_tree::time::Uptime;
use tracing_tree::HierarchicalLayer;
pub fn setup_logging(tracing: bool) {
    let default_level = if tracing { "preco=trace" } else { "preco=info" };
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_level))
        .unwrap();

    if tracing {
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
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_writer(std::io::stderr),
            )
            .init();
    }
}
