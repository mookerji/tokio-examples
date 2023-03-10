use crate::types::*;

use opentelemetry;
use prost::Message;
use rand::prelude::*;
use redis::Commands;
use service as proto;
use service::key_value_server::KeyValue;
use service::key_value_server::KeyValueServer;
use service::measurement_server::Measurement;
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::pin::Pin;
use std::str::from_utf8;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use tonic_health::server::HealthReporter;
use zmq;

use opentelemetry::{
    propagation::Extractor,
    trace::{Span, Tracer},
};

// PUll in definitions

pub mod service {
    tonic::include_proto!("service");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("service_descriptor");

// Utilities for metadata
//
// Copied from:
// https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/grpc/src/server.rs

struct MetadataMap<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataMap<'a> {
    /// Get a value for a key from the MetadataMap.  If the value can't be converted to &str, returns None
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    /// Collect all the keys from the MetadataMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

// Service definition

pub struct KeyValueApp {
    // Handles to redis backing service
    redis_client: Arc<Mutex<redis::Client>>,
    redis_conn: Arc<Mutex<redis::Connection>>,

    // Handle for updating serving status
    health_reporter: HealthReporter,
}

impl KeyValueApp {
    pub fn new(health_reporter: HealthReporter) -> Result<KeyValueApp> {
        let hostname = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
        let client = redis::Client::open(format!("redis://{}/", hostname))?;
        let mut conn = client.get_connection()?;
        Ok(KeyValueApp {
            redis_client: Arc::new(Mutex::new(client)),
            redis_conn: Arc::new(Mutex::new(conn)),
            health_reporter: health_reporter,
        })
    }

    // NOTE: Ideally this would be a separate task that periodically checked
    // internal state.
    pub async fn set_serving(&mut self) {
        self.health_reporter
            .set_serving::<KeyValueServer<KeyValueApp>>()
            .await;
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
        let parent_cx = opentelemetry::global::get_text_map_propagator(|prop| {
            prop.extract(&MetadataMap(request.metadata()))
        });
        let mut span = opentelemetry::global::tracer("key-value")
            .start_with_context("Processing reply", &parent_cx);
        span.set_attribute(opentelemetry::KeyValue::new(
            "request",
            format!("{request:?}"),
        ));
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

fn random() -> f32 {
    rand::thread_rng().gen()
}

#[derive(Debug)]
pub struct MeasurementApp {
    tx_buf: broadcast::Sender<proto::MeasurementResponse>,
}

impl MeasurementApp {
    pub async fn new() -> MeasurementApp {
        let (tx, _) = broadcast::channel(16);
        let tx2 = tx.clone();
        let counter = AtomicI32::new(1);
        thread::spawn(move || loop {
            tracing::info!("Polling!");
            tx2.send(proto::MeasurementResponse {
                data: random(),
                counter: counter.fetch_add(1, Ordering::SeqCst),
            });
            std::thread::sleep(Duration::from_millis(1000));
        });
        MeasurementApp {
            tx_buf: tx,
        }
    }
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
            counter: -1,
        }))
    }

    type ReadMeasurementsStream = Pin<
        Box<
            dyn Stream<Item = tonic::Result<proto::MeasurementResponse, Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn read_measurements(
        &self,
        request: tonic::Request<proto::MeasurementRequest>,
    ) -> tonic::Result<tonic::Response<Self::ReadMeasurementsStream>, tonic::Status> {
        tracing::info!("received request / sent response");
        let stream = BroadcastStream::new(self.tx_buf.subscribe());
        let output = stream.filter_map(|res| res.ok()).map(Ok);
        Ok(Response::new(
            std::boxed::Box::pin(output) as Self::ReadMeasurementsStream
        ))
    }
}

// NOTE(mookerji): What?
//
// tonic-grpc-modbus-service-1       | INFO:root:Starting pymodbus server
// tonic-grpc-measurement-service-1  | 2023-02-27T06:57:46.539641Z  INFO ThreadId(06) serviced: src/main.rs:32: Starting server. addr=0.0.0.0:50052
// tonic-grpc-measurement-service-1  | 2023-02-27T06:57:46.541277Z  INFO ThreadId(08) tokio_example::app: src/app.rs:147: Sleeper running and returning to sleep. Elapsed=2.911158ms
// tonic-grpc-measurement-service-1  | 2023-02-27T06:57:46.539750Z  INFO ThreadId(07) tokio_example::app: src/app.rs:170: Started ZMQ backend
// tonic-grpc-key-value-service-1    | 2023-02-27T06:57:46.555549Z  INFO ThreadId(08) tokio_example::app: src/app.rs:147: Sleeper running and returning to sleep. Elapsed=720.743µs
pub fn run_sleeper() -> std::thread::JoinHandle<()> {
    let start = std::time::Instant::now();
    thread::spawn(move || loop {
        tracing::info!(
            "Sleeper running and returning to sleep. Elapsed={:?}",
            start.elapsed()
        );
        std::thread::sleep(Duration::from_millis(10000));
    })
}

pub fn run_zeromq() -> std::thread::JoinHandle<()> {
    thread::spawn(|| {
        let context = zmq::Context::new();
        let frontend = context.socket(zmq::SUB).unwrap();
        frontend
            .connect("tcp://localhost:5557")
            .expect("could not connect to frontend");
        let backend = context.socket(zmq::XPUB).unwrap();
        backend
            .bind("tcp://*:5558")
            .expect("could not bind backend socket");
        //  Subscribe to every single topic from publisher
        frontend.set_subscribe(b"").unwrap();
        let mut msg = zmq::Message::new();
        let mut cache = HashMap::new();
        tracing::info!("Started ZMQ backend");
        loop {
            let mut items = [
                frontend.as_poll_item(zmq::POLLIN),
                backend.as_poll_item(zmq::POLLIN),
            ];
            if zmq::poll(&mut items, 10000).is_err() {
                tracing::info!("ZMQ poll timeout");
                break;
            }
            if items[0].is_readable() {
                let topic = frontend.recv_msg(0).unwrap();
                let current = frontend.recv_msg(0).unwrap();
                cache.insert(topic.to_vec(), current.to_vec());
                backend.send(topic, zmq::SNDMORE).unwrap();
                backend.send(current, 0).unwrap();
            }
            if items[1].is_readable() {
                let event = backend.recv_msg(0).unwrap();
                if event[0] == 1 {
                    let topic = &event[1..];
                    tracing::info!("Sending cached topic {}", from_utf8(topic).unwrap());
                    if let Some(previous) = cache.get(topic) {
                        backend.send(topic, zmq::SNDMORE).unwrap();
                        backend.send(previous, 0).unwrap();
                    }
                }
            }
        }
    })
}
