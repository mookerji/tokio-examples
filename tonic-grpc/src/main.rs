use service::key_value_server::KeyValueServer;
use service::measurement_server::MeasurementServer;
use std::env;
use tokio_example::app::*;
use tokio_example::types::*;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

// Moves the Tonic runtime to a separate thread, based on
// https://tokio.rs/tokio/topics/bridging#sending-messages. This partitions an
// async portion of a service from an existing non-async service runtime.
//
// TODO(mookerji): replace u8 with proto::KeyValueWriteResponse to show interop
// between async/sync backends

#[derive(Clone)]
pub struct GrpcContainer {
    channel: tokio::sync::mpsc::Sender<u8>,
}

impl GrpcContainer {
    pub fn new() -> Result<GrpcContainer> {
        let (send, mut recv) = tokio::sync::mpsc::channel(16);
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        std::thread::spawn(move || {
            rt.block_on(async move {
                let addr = env::var("GRPC_ADDR")
                    .unwrap_or("0.0.0.0:50051".to_string())
                    .parse()?;
                let reflection_service = tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                    .build()?;
                tracing::info!(message = "Starting server.", %addr);
                Server::builder()
                    .trace_fn(|_| tracing::info_span!("keyvalue_server"))
                    .accept_http1(true)
                    .layer(GrpcWebLayer::new())
                    .add_service(reflection_service)
                    .add_service(KeyValueServer::new(KeyValueApp::new()?))
                    .add_service(MeasurementServer::new(MeasurementApp::default()))
                    .serve(addr)
                    .await?;
                Ok::<(), Box<dyn std::error::Error>>(())
            });
        });
        Ok(GrpcContainer { channel: send })
    }

    pub fn send(&self, item: u8) {
        match self.channel.blocking_send(item) {
            Ok(()) => {}
            Err(_) => panic!("The shared runtime has shut down."),
        }
    }
}

fn main() -> Result<()> {
    let format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .compact();
    tracing_subscriber::fmt().event_format(format).init();
    let grpc = GrpcContainer::new()?;
    let threads = vec![run_zeromq(), run_sleeper()];
    for handle in threads {
        handle.join().unwrap();
    }
    Ok(())
}
