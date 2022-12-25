use service as proto;
use service::key_value_client::KeyValueClient;
use service::KeyValueWriteRequest;
use std::env;
use tokio_example::types::*;
use tokio_modbus::prelude::*;

use std::net::{SocketAddr, ToSocketAddrs};

pub mod service {
    tonic::include_proto!("service");
}

fn lookup(endpoint: String) -> Result<SocketAddr> {
    let mut addrs_iter = endpoint.to_socket_addrs()?;
    Ok(addrs_iter.next().unwrap())
}

// Mocked Sunspec common block. See tools/python/service_modbus.py.
const REGISTER_BLOCK_OFFSET: u16 = 40000;
const REGISTER_BLOCK_LENGTH: u16 = 69;

#[tokio::main]
async fn main() -> Result<()> {
    let format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .compact();
    tracing_subscriber::fmt().event_format(format).init();
    let modbus_address = lookup(env::var("MODBUS_ADDR").unwrap_or("localhost:502".to_string()))?;
    let mut modbus_client = tcp::connect(modbus_address).await?;
    tracing::info!("starting poll {:?}", modbus_address);
    let grpc_uri = env::var("GRPC_ADDR").unwrap_or("http://localhost:50051".to_string());
    let mut grpc_client = KeyValueClient::connect(grpc_uri).await?;
    tracing::info!("starting grpc client {:?}", grpc_client);
    loop {
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
        let request = tonic::Request::new(write_request);
        let grpc_response = grpc_client.write_key_value(request).await?;
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
