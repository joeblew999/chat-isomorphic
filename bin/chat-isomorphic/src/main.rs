use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let backends = compiled_backends();
    tracing::info!(?backends, "chat-isomorphic starting");
    if backends.is_empty() {
        tracing::warn!(
            "no backends compiled in — build with --features signal (or another) to enable one"
        );
    }
    Ok(())
}

fn compiled_backends() -> Vec<&'static str> {
    #[allow(unused_mut)]
    let mut v = Vec::new();
    #[cfg(feature = "signal")]
    v.push("signal");
    v
}
