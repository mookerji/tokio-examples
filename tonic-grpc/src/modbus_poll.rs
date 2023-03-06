use opentelemetry;
use opentelemetry::global;

// use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::TraceResult;
use opentelemetry::{
    propagation::Injector,
    sdk::trace::Tracer,
    trace::{TraceContextExt, Tracer as _},
};

use service as proto;
use service::key_value_client::KeyValueClient;
use std::env;
use tokio_example::types::*;
use tokio_example::utils::*;
use tokio_modbus::prelude::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use std::net::{SocketAddr, ToSocketAddrs};

pub mod service {
    tonic::include_proto!("service");
}

// Utilities for metadata
//
// Copied from:
// https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/grpc/src/client.rs

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::from_str(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

fn lookup(endpoint: String) -> Result<SocketAddr> {
    let mut addrs_iter = endpoint.to_socket_addrs()?;
    Ok(addrs_iter.next().unwrap())
}

// Mocked Sunspec common block. See tools/python/service_modbus.py.
const REGISTER_BLOCK_OFFSET: u16 = 40000;
const REGISTER_BLOCK_LENGTH: u16 = 69;

fn init_logging() -> TraceResult<Tracer> {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let jaeger_endpoint = env::var("JAEGER_ENDPOINT").unwrap_or("localhost:6831".to_string());
    opentelemetry_jaeger::new_pipeline()
        .with_service_name(bin_name().unwrap_or("UNKNOWN_SERVICE".to_string()))
        .with_agent_endpoint(jaeger_endpoint)
        .install_simple()
    // let format = fmt::Layer::default()
    //     .with_file(true)
    //     .with_line_number(true)
    //     .with_level(true)
    //     .with_target(true)
    //     .with_thread_ids(true)
    //     .with_thread_names(true)
    //     .compact();
    // tracing_subscriber::registry()
    //     .with(opentelem)
    //     .with(format)
    //     .try_init().unwrap();
    // Ok(tracer)
}

#[tokio::main]
async fn main() -> Result<()> {
    let tracer = init_logging()?;
    let modbus_address = lookup(env::var("MODBUS_ADDR").unwrap_or("0.0.0.0:502".to_string()))?;
    println!("here! {:?}", modbus_address);
    let mut modbus_client = tcp::connect(modbus_address).await?;
    tracing::info!("starting poll {:?}", modbus_address);
    let grpc_uri = env::var("GRPC_ADDR").unwrap_or("http://localhost:50051".to_string());
    let mut grpc_client = KeyValueClient::connect(grpc_uri).await?;
    tracing::info!("starting grpc client {:?}", grpc_client);
    loop {
        let span = tracer.start("client-request");
        let cx = opentelemetry::Context::current_with_span(span);

        let mut write_request = proto::KeyValueWriteRequest::default();
        let data = modbus_client
            .read_input_registers(REGISTER_BLOCK_OFFSET, REGISTER_BLOCK_LENGTH)
            .await?;
        for (register, value) in data.iter().enumerate() {
            write_request.items.push(proto::KeyValueItem {
                key: format!("modbus/device=mock/register={}", register.to_string()),
                value: Some(proto::key_value_item::Value::IntValue(*value as i32)),
            });
        }
        let mut request = tonic::Request::new(write_request);

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut MetadataMap(request.metadata_mut()))
        });

        let grpc_response = grpc_client.write_key_value(request).await?;

        cx.span().add_event(
            "response-received".to_string(),
            vec![opentelemetry::KeyValue::new(
                "response",
                format!("{grpc_response:?}"),
            )],
        );

        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
    //shutdown_tracer_provider();
}
