use service::key_value_server::KeyValueServer;
use std::env;
use tokio_example::app::*;
use tokio_example::types::*;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_level(false)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .compact();
    tracing_subscriber::fmt().event_format(format).init();
    let app = App::new()?;
    let addr = env::var("GRPC_ADDR")
        .unwrap_or("0.0.0.0:50051".to_string())
        .parse()?;
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;
    tracing::info!(message = "Starting server.", %addr);
    Server::builder()
        .trace_fn(|_| tracing::info_span!("keyvalue_server"))
        .add_service(reflection_service)
        .add_service(KeyValueServer::new(app))
        .serve(addr)
        .await?;
    Ok(())
}
