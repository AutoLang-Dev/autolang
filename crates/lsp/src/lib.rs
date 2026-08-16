use crate::server::Server;
use async_lsp::{
  MainLoop, client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
  panic::CatchUnwindLayer, server::LifecycleLayer, tracing::TracingLayer,
};
use tower::ServiceBuilder;

mod server;

async fn run_async() -> async_lsp::Result<()> {
  tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .with_ansi(false)
    .init();

  let (server, _) = MainLoop::new_server(|client| {
    ServiceBuilder::new()
      .layer(TracingLayer::default())
      .layer(LifecycleLayer::default())
      .layer(CatchUnwindLayer::default())
      .layer(ConcurrencyLayer::default())
      .layer(ClientProcessMonitorLayer::new(client.clone()))
      .service(Server::router(client))
  });

  #[cfg(unix)]
  let (stdin, stdout) = (
    async_lsp::stdio::PipeStdin::lock_tokio()?,
    async_lsp::stdio::PipeStdout::lock_tokio()?,
  );

  #[cfg(not(unix))]
  let (stdin, stdout) = (
    tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
    tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
  );

  server.run_buffered(stdin, stdout).await
}

pub fn run() -> anyhow::Result<()> {
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .build()?;

  runtime.block_on(run_async())?;
  Ok(())
}
