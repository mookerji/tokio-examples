use crate::types::*;
use redis::Commands;
use service as proto;
use service::key_value_server::KeyValue;
use std::env;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

pub mod service {
    tonic::include_proto!("service");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("service_descriptor");

pub struct App {
    redis_client: Arc<Mutex<redis::Client>>,
    redis_conn: Arc<Mutex<redis::Connection>>,
}

impl App {
    pub fn new() -> Result<App> {
        let hostname = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
        let client = redis::Client::open(format!("redis://{}/", hostname))?;
        let mut conn = client.get_connection()?;
        Ok(App {
            redis_client: Arc::new(Mutex::new(client)),
            redis_conn: Arc::new(Mutex::new(conn)),
        })
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "App")
    }
}

#[tonic::async_trait]
impl KeyValue for App {
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
                let val: redis::RedisResult<String> = conn.get(&key);
                reply.items.push(proto::KeyValueItem {
                    key: key,
                    value: val.unwrap_or("MISSING".to_string()),
                });
            }
            reply.status = proto::Status::Ok as i32;
        } else {
            reply.status = proto::Status::ErrorInternal as i32;
        }
        tracing::info!("sent request");
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
                let val: redis::RedisResult<String> = conn.set(&item.key, item.value);
            }
            reply.status = proto::Status::Ok as i32;
        } else {
            reply.status = proto::Status::ErrorInternal as i32;
        }
        tracing::info!("sent request");
        Ok(Response::new(reply))
    }
}
