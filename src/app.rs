use crate::types::*;

use prost::Message;
use rand::prelude::*;
use redis::Commands;
use service as proto;
use service::key_value_server::KeyValue;
use service::measurement_server::Measurement;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub mod service {
    tonic::include_proto!("service");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("service_descriptor");

pub struct KeyValueApp {
    redis_client: Arc<Mutex<redis::Client>>,
    redis_conn: Arc<Mutex<redis::Connection>>,
}

impl KeyValueApp {
    pub fn new() -> Result<KeyValueApp> {
        let hostname = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
        let client = redis::Client::open(format!("redis://{}/", hostname))?;
        let mut conn = client.get_connection()?;
        Ok(KeyValueApp {
            redis_client: Arc::new(Mutex::new(client)),
            redis_conn: Arc::new(Mutex::new(conn)),
        })
    }
}

impl std::fmt::Debug for KeyValueApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyValueApp")
    }
}

#[tonic::async_trait]
impl KeyValue for KeyValueApp {
    #[tracing::instrument]
    async fn read_key_value(
        &self,
        request: Request<proto::KeyValueReadRequest>,
    ) -> tonic::Result<Response<proto::KeyValueReadResponse>, Status> {
        tracing::info!("received request");
        let conn = self.redis_conn.clone();
        let mut reply = proto::KeyValueReadResponse::default();
        let read_request = request.into_inner();
        if let Ok(mut conn) = self.redis_conn.clone().lock() {
            for key in read_request.keys {
                let val: redis::RedisResult<Vec<u8>> = conn.get(&key);
                if let Ok(buf) = val {
                    let mut y = std::io::Cursor::new(buf);
                    reply.items.push(
                        proto::KeyValueItem::decode(&mut y)
                            .unwrap_or(proto::KeyValueItem::default()),
                    );
                } else {
                    tracing::error!("Decode error");
                }
            }
            reply.status = proto::Status::Ok as i32;
        } else {
            reply.status = proto::Status::ErrorInternal as i32;
        }
        // NOTE: status handling is broken
        tracing::info!("sent response");
        Ok(Response::new(reply))
    }

    #[tracing::instrument]
    async fn write_key_value(
        &self,
        request: Request<proto::KeyValueWriteRequest>,
    ) -> tonic::Result<Response<proto::KeyValueWriteResponse>, Status> {
        tracing::info!("received request");
        let conn = self.redis_conn.clone();
        let mut reply = proto::KeyValueWriteResponse::default();
        let write_request = request.into_inner();
        if let Ok(mut conn) = self.redis_conn.clone().lock() {
            for item in write_request.items {
                let val: redis::RedisResult<Vec<u8>> = conn.set(&item.key, item.encode_to_vec());
            }
            reply.status = proto::Status::Ok as i32;
        } else {
            reply.status = proto::Status::ErrorInternal as i32;
        }
        tracing::info!("sent response");
        Ok(Response::new(reply))
    }
}

#[derive(Debug, Default)]
pub struct MeasurementApp {}

async fn random() -> f32 {
    rand::thread_rng().gen()
}

#[tonic::async_trait]
impl Measurement for MeasurementApp {
    #[tracing::instrument]
    async fn read_measurement(
        &self,
        request: Request<proto::MeasurementRequest>,
    ) -> tonic::Result<Response<proto::MeasurementResponse>, Status> {
        tracing::info!("received request / sent response");
        Ok(Response::new(proto::MeasurementResponse {
            data: rand::thread_rng().gen(),
        }))
    }

    type ReadMeasurementsStream = ReceiverStream<tonic::Result<proto::MeasurementResponse, Status>>;

    async fn read_measurements(
        &self,
        request: tonic::Request<proto::MeasurementRequest>,
    ) -> tonic::Result<tonic::Response<Self::ReadMeasurementsStream>, tonic::Status> {
        let (mut tx, rx) = mpsc::channel(4);
        tracing::info!("received request / sent response");
        // TODO: figure out how this works
        tokio::spawn(async move {
            tx.send(Ok(proto::MeasurementResponse {
                data: random().await,
            }))
            .await
            .unwrap();
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
