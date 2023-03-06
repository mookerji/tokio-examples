# Tonic Async Exploration

Demo's Rust-based grpc services using Tonic and the Tokio async runtime.

## Development

Start app:

```
make
```

Install dependencies:

```
$ make deps
```

## Clients

```
$  grpcurl -plaintext -d '{"items": [{"key": "foo", "string_value": "bar"}]}' '127.0.0.1:50051' service.KeyValue/WriteKeyValue
$  grpcurl -plaintext -d '{"keys": ["foo"]}' '127.0.0.1:50051' service.KeyValue/ReadKeyValue
{
  "items": [
    {
      "key": "foo",
      "stringValue": "bar"
    }
  ]
}
```

## Usage

### Without Docker

Start dependent services:

```bash
// $ docker run --name some-redis redis
$ brew services start redis
$ docker run -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
```

```bash
$ RUST_BACKTRACE=1 cargo run --bin serviced
```

```bash
$ grpcurl -plaintext -d '{"items": [{"key": "foo", "string_value": "bar"}]}' '127.0.0.1:50051' service.KeyValue/WriteKeyValue
```

```bash
$ python3 tools/python/serve_modbus.py
$ MODBUS_ADDR='0.0.0.0:502' RUST_BACKTRACE=1 cargo run --bin modbus-poll
```

## Bugs

### Tonic creates IPv6 socket to communicate with IPv4 endpoint (?)

The Tonic grpc client creates a IPv6 socket (?) to communicate with a grpc
server that (I believe) is only listening over IPv4. Docker for Mac doesn't
support IPv6, so grpc client initialization to another client fails. It's not
apparently not possible to do container-to-container grpc communication with
Docker on Mac for this reason.

See also:

- https://docs.docker.com/config/daemon/ipv6/
- https://github.com/docker/for-mac/issues/1432

With stack traces:

```
# strace -k -e trace=network modbus-poll

2022-12-25T18:41:07.531622Z  INFO main ThreadId(01) modbus_poll: src/modbus_poll.rs:36: starting poll 172.24.0.2:502
socket(AF_INET, SOCK_STREAM|SOCK_CLOEXEC, IPPROTO_TCP) = 10
 > /lib/x86_64-linux-gnu/libc-2.28.so(socket+0x7) [0xfa667]
 > /cache/cargo-home/bin/modbus-poll(socket2::socket::Socket::new+0x1d) [0x22407d]
 > /cache/cargo-home/bin/modbus-poll(hyper::client::connect::http::connect+0x69) [0x174749]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x57e) [0xe0c6e]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x6d3) [0xd82a3]
 > /cache/cargo-home/bin/modbus-poll(_ZN97_$LT$core..future..from_generator..GenFuture$LT$T$GT$$u20$as$u20$core..future..future..Future$GT$4poll17h642f5a7c90e7ccd7E.llvm.611427016915381354+0x54) [0xd79d4]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x76) [0xdc106]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::reconnect::Reconnect<M,Target> as tower_service::Service<Request>>::poll_ready+0x879) [0xa0679]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::either::Either<A,B> as tower_service::Service<Request>>::poll_ready+0x2f) [0x6a20f]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::map_future::MapFuture<S,F> as tower_service::Service<R>>::poll_ready+0x1f) [0x6b2bf]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::connection::Connection as tower_service::Service<http::request::Request<http_body::combinators::box_body::UnsyncBoxBody<bytes::bytes::Bytes,tonic::status::Status>>>>::poll_ready+0x1a) [0x162a7a]
 > /cache/cargo-home/bin/modbus-poll(_ZN11modbus_poll4main28_$u7b$$u7b$closure$u7d$$u7d$17hab0dbe17d0dca802E.llvm.611427016915381354+0x235d) [0xe416d]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::park::CachedParkThread::block_on+0xe1) [0xad2d1]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::scheduler::multi_thread::MultiThread::block_on+0x70) [0x108030]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::runtime::Runtime::block_on+0x5d) [0x12790d]
 > /cache/cargo-home/bin/modbus-poll(modbus_poll::main+0xc2) [0x8d272]
 > /cache/cargo-home/bin/modbus-poll(std::sys_common::backtrace::__rust_begin_short_backtrace+0x3) [0x121db3]
 > /cache/cargo-home/bin/modbus-poll(_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h2926171c311272bdE.llvm.1449145217282581913+0x9) [0xa45d9]
 > /cache/cargo-home/bin/modbus-poll(std::rt::lang_start_internal+0x42e) [0x23ba7e]
 > /cache/cargo-home/bin/modbus-poll(main+0x22) [0x8d352]
 > /lib/x86_64-linux-gnu/libc-2.28.so(__libc_start_main+0xeb) [0x2409b]
 > /cache/cargo-home/bin/modbus-poll(_start+0x2a) [0x4965a]
connect(10, {sa_family=AF_INET, sin_port=htons(50051), sin_addr=inet_addr("127.0.0.1")}, 16) = -1 EINPROGRESS (Operation now in progress)
 > /lib/x86_64-linux-gnu/libpthread-2.28.so(connect+0x47) [0x11707]
 > /cache/cargo-home/bin/modbus-poll(socket2::socket::Socket::connect+0x2d) [0x22416d]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x6c) [0xd6bac]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x79c) [0xe0e8c]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x6d3) [0xd82a3]
 > /cache/cargo-home/bin/modbus-poll(_ZN97_$LT$core..future..from_generator..GenFuture$LT$T$GT$$u20$as$u20$core..future..future..Future$GT$4poll17h642f5a7c90e7ccd7E.llvm.611427016915381354+0x54) [0xd79d4]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x76) [0xdc106]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::reconnect::Reconnect<M,Target> as tower_service::Service<Request>>::poll_ready+0x879) [0xa0679]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::either::Either<A,B> as tower_service::Service<Request>>::poll_ready+0x2f) [0x6a20f]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::map_future::MapFuture<S,F> as tower_service::Service<R>>::poll_ready+0x1f) [0x6b2bf]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::connection::Connection as tower_service::Service<http::request::Request<http_body::combinators::box_body::UnsyncBoxBody<bytes::bytes::Bytes,tonic::status::Status>>>>::poll_ready+0x1a) [0x162a7a]
 > /cache/cargo-home/bin/modbus-poll(_ZN11modbus_poll4main28_$u7b$$u7b$closure$u7d$$u7d$17hab0dbe17d0dca802E.llvm.611427016915381354+0x235d) [0xe416d]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::park::CachedParkThread::block_on+0xe1) [0xad2d1]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::scheduler::multi_thread::MultiThread::block_on+0x70) [0x108030]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::runtime::Runtime::block_on+0x5d) [0x12790d]
 > /cache/cargo-home/bin/modbus-poll(modbus_poll::main+0xc2) [0x8d272]
 > /cache/cargo-home/bin/modbus-poll(std::sys_common::backtrace::__rust_begin_short_backtrace+0x3) [0x121db3]
 > /cache/cargo-home/bin/modbus-poll(_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h2926171c311272bdE.llvm.1449145217282581913+0x9) [0xa45d9]
 > /cache/cargo-home/bin/modbus-poll(std::rt::lang_start_internal+0x42e) [0x23ba7e]
 > /cache/cargo-home/bin/modbus-poll(main+0x22) [0x8d352]
 > /lib/x86_64-linux-gnu/libc-2.28.so(__libc_start_main+0xeb) [0x2409b]
 > /cache/cargo-home/bin/modbus-poll(_start+0x2a) [0x4965a]
getsockopt(10, SOL_SOCKET, SO_ERROR, [ECONNREFUSED], [4]) = 0
 > /lib/x86_64-linux-gnu/libc-2.28.so(getsockopt+0xa) [0xfa16a]
 > /cache/cargo-home/bin/modbus-poll(std::net::tcp::TcpListener::take_error+0x2a) [0x24262a]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0xbc) [0xe05cc]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0xf2) [0xd6c32]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x79c) [0xe0e8c]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x6d3) [0xd82a3]
 > /cache/cargo-home/bin/modbus-poll(_ZN97_$LT$core..future..from_generator..GenFuture$LT$T$GT$$u20$as$u20$core..future..future..Future$GT$4poll17h642f5a7c90e7ccd7E.llvm.611427016915381354+0x54) [0xd79d4]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x76) [0xdc106]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::reconnect::Reconnect<M,Target> as tower_service::Service<Request>>::poll_ready+0x879) [0xa0679]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::either::Either<A,B> as tower_service::Service<Request>>::poll_ready+0x2f) [0x6a20f]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::map_future::MapFuture<S,F> as tower_service::Service<R>>::poll_ready+0x1f) [0x6b2bf]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::connection::Connection as tower_service::Service<http::request::Request<http_body::combinators::box_body::UnsyncBoxBody<bytes::bytes::Bytes,tonic::status::Status>>>>::poll_ready+0x1a) [0x162a7a]
 > /cache/cargo-home/bin/modbus-poll(_ZN11modbus_poll4main28_$u7b$$u7b$closure$u7d$$u7d$17hab0dbe17d0dca802E.llvm.611427016915381354+0x235d) [0xe416d]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::park::CachedParkThread::block_on+0xe1) [0xad2d1]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::scheduler::multi_thread::MultiThread::block_on+0x70) [0x108030]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::runtime::Runtime::block_on+0x5d) [0x12790d]
 > /cache/cargo-home/bin/modbus-poll(modbus_poll::main+0xc2) [0x8d272]
 > /cache/cargo-home/bin/modbus-poll(std::sys_common::backtrace::__rust_begin_short_backtrace+0x3) [0x121db3]
 > /cache/cargo-home/bin/modbus-poll(_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h2926171c311272bdE.llvm.1449145217282581913+0x9) [0xa45d9]
 > /cache/cargo-home/bin/modbus-poll(std::rt::lang_start_internal+0x42e) [0x23ba7e]
 > /cache/cargo-home/bin/modbus-poll(main+0x22) [0x8d352]
 > /lib/x86_64-linux-gnu/libc-2.28.so(__libc_start_main+0xeb) [0x2409b]
 > /cache/cargo-home/bin/modbus-poll(_start+0x2a) [0x4965a]
socket(AF_INET6, SOCK_STREAM|SOCK_CLOEXEC, IPPROTO_TCP) = 10
 > /lib/x86_64-linux-gnu/libc-2.28.so(socket+0x7) [0xfa667]
 > /cache/cargo-home/bin/modbus-poll(socket2::socket::Socket::new+0x1d) [0x22407d]
 > /cache/cargo-home/bin/modbus-poll(hyper::client::connect::http::connect+0x69) [0x174749]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x57e) [0xe0c6e]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0xb17) [0xd86e7]
 > /cache/cargo-home/bin/modbus-poll(_ZN97_$LT$core..future..from_generator..GenFuture$LT$T$GT$$u20$as$u20$core..future..future..Future$GT$4poll17h642f5a7c90e7ccd7E.llvm.611427016915381354+0x54) [0xd79d4]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x76) [0xdc106]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::reconnect::Reconnect<M,Target> as tower_service::Service<Request>>::poll_ready+0x879) [0xa0679]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::either::Either<A,B> as tower_service::Service<Request>>::poll_ready+0x2f) [0x6a20f]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::map_future::MapFuture<S,F> as tower_service::Service<R>>::poll_ready+0x1f) [0x6b2bf]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::connection::Connection as tower_service::Service<http::request::Request<http_body::combinators::box_body::UnsyncBoxBody<bytes::bytes::Bytes,tonic::status::Status>>>>::poll_ready+0x1a) [0x162a7a]
 > /cache/cargo-home/bin/modbus-poll(_ZN11modbus_poll4main28_$u7b$$u7b$closure$u7d$$u7d$17hab0dbe17d0dca802E.llvm.611427016915381354+0x235d) [0xe416d]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::park::CachedParkThread::block_on+0xe1) [0xad2d1]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::scheduler::multi_thread::MultiThread::block_on+0x70) [0x108030]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::runtime::Runtime::block_on+0x5d) [0x12790d]
 > /cache/cargo-home/bin/modbus-poll(modbus_poll::main+0xc2) [0x8d272]
 > /cache/cargo-home/bin/modbus-poll(std::sys_common::backtrace::__rust_begin_short_backtrace+0x3) [0x121db3]
 > /cache/cargo-home/bin/modbus-poll(_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h2926171c311272bdE.llvm.1449145217282581913+0x9) [0xa45d9]
 > /cache/cargo-home/bin/modbus-poll(std::rt::lang_start_internal+0x42e) [0x23ba7e]
 > /cache/cargo-home/bin/modbus-poll(main+0x22) [0x8d352]
 > /lib/x86_64-linux-gnu/libc-2.28.so(__libc_start_main+0xeb) [0x2409b]
 > /cache/cargo-home/bin/modbus-poll(_start+0x2a) [0x4965a]
connect(10, {sa_family=AF_INET6, sin6_port=htons(50051), inet_pton(AF_INET6, "::1", &sin6_addr), sin6_flowinfo=htonl(0), sin6_scope_id=0}, 28) = -1 EADDRNOTAVAIL (Cannot assign requested address)
 > /lib/x86_64-linux-gnu/libpthread-2.28.so(connect+0x47) [0x11707]
 > /cache/cargo-home/bin/modbus-poll(socket2::socket::Socket::connect+0x2d) [0x22416d]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x6c) [0xd6bac]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x79c) [0xe0e8c]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0xb17) [0xd86e7]
 > /cache/cargo-home/bin/modbus-poll(_ZN97_$LT$core..future..from_generator..GenFuture$LT$T$GT$$u20$as$u20$core..future..future..Future$GT$4poll17h642f5a7c90e7ccd7E.llvm.611427016915381354+0x54) [0xd79d4]
 > /cache/cargo-home/bin/modbus-poll(<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll+0x76) [0xdc106]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::reconnect::Reconnect<M,Target> as tower_service::Service<Request>>::poll_ready+0x879) [0xa0679]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::either::Either<A,B> as tower_service::Service<Request>>::poll_ready+0x2f) [0x6a20f]
 > /cache/cargo-home/bin/modbus-poll(<tower::util::map_future::MapFuture<S,F> as tower_service::Service<R>>::poll_ready+0x1f) [0x6b2bf]
 > /cache/cargo-home/bin/modbus-poll(<tonic::transport::service::connection::Connection as tower_service::Service<http::request::Request<http_body::combinators::box_body::UnsyncBoxBody<bytes::bytes::Bytes,tonic::status::Status>>>>::poll_ready+0x1a) [0x162a7a]
 > /cache/cargo-home/bin/modbus-poll(_ZN11modbus_poll4main28_$u7b$$u7b$closure$u7d$$u7d$17hab0dbe17d0dca802E.llvm.611427016915381354+0x235d) [0xe416d]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::park::CachedParkThread::block_on+0xe1) [0xad2d1]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::scheduler::multi_thread::MultiThread::block_on+0x70) [0x108030]
 > /cache/cargo-home/bin/modbus-poll(tokio::runtime::runtime::Runtime::block_on+0x5d) [0x12790d]
 > /cache/cargo-home/bin/modbus-poll(modbus_poll::main+0xc2) [0x8d272]
 > /cache/cargo-home/bin/modbus-poll(std::sys_common::backtrace::__rust_begin_short_backtrace+0x3) [0x121db3]
 > /cache/cargo-home/bin/modbus-poll(_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h2926171c311272bdE.llvm.1449145217282581913+0x9) [0xa45d9]
 > /cache/cargo-home/bin/modbus-poll(std::rt::lang_start_internal+0x42e) [0x23ba7e]
 > /cache/cargo-home/bin/modbus-poll(main+0x22) [0x8d352]
 > /lib/x86_64-linux-gnu/libc-2.28.so(__libc_start_main+0xeb) [0x2409b]
 > /cache/cargo-home/bin/modbus-poll(_start+0x2a) [0x4965a]
Error: tonic::transport::Error(Transport, hyper::Error(Connect, ConnectError("tcp connect error", Os { code: 99, kind: AddrNotAvailable, message: "Cannot assign requested address" })))
```

Simplified:

```
# strace -e trace=network modbus-poll
...
2022-12-25T19:21:42.814822Z  INFO main ThreadId(01) modbus_poll: src/modbus_poll.rs:36: starting poll 172.25.0.3:502
socket(AF_INET, SOCK_STREAM|SOCK_CLOEXEC, IPPROTO_TCP) = 10
connect(10, {sa_family=AF_INET, sin_port=htons(50051), sin_addr=inet_addr("127.0.0.1")}, 16) = -1 EINPROGRESS (Operation now in progress)
getsockopt(10, SOL_SOCKET, SO_ERROR, [ECONNREFUSED], [4]) = 0
socket(AF_INET6, SOCK_STREAM|SOCK_CLOEXEC, IPPROTO_TCP) = 10
connect(10, {sa_family=AF_INET6, sin6_port=htons(50051), inet_pton(AF_INET6, "::1", &sin6_addr), sin6_flowinfo=htonl(0), sin6_scope_id=0}, 28) = -1 EADDRNOTAVAIL (Cannot assign requested address)
Error: tonic::transport::Error(Transport, hyper::Error(Connect, ConnectError("tcp connect error", Os { code: 99, kind: AddrNotAvailable, message: "Cannot assign requested address" })))
```

Compare to:

```
# strace -e trace=network ./grpcurl -plaintext -d '{"keys": ["foo"]}' 'key-value-service:50051' service.KeyValue/ReadKeyValue
socket(AF_INET, SOCK_DGRAM|SOCK_CLOEXEC|SOCK_NONBLOCK, IPPROTO_IP) = 7
setsockopt(7, SOL_SOCKET, SO_BROADCAST, [1], 4) = 0
connect(7, {sa_family=AF_INET, sin_port=htons(53), sin_addr=inet_addr("127.0.0.11")}, 16) = 0
getsockname(7, {sa_family=AF_INET, sin_port=htons(46111), sin_addr=inet_addr("127.0.0.1")}, [112->16]) = 0
getpeername(7, {sa_family=AF_INET, sin_port=htons(53), sin_addr=inet_addr("127.0.0.11")}, [112->16]) = 0
socket(AF_INET, SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK, IPPROTO_IP) = 3
connect(3, {sa_family=AF_INET, sin_port=htons(50051), sin_addr=inet_addr("172.25.0.5")}, 16) = -1 EINPROGRESS (Operation now in progress)
{
  "items": [
    {
      "key": "foo",
      "stringValue": "bar"
    }
  ]
}
+++ exited with 0 +++
```

## OpenTelemetry not communicating with Jaeger

```
$ make
```

```
$ grpcurl -plaintext -d '{"items": [{"key": "foo", "string_value": "bar"}]}' '127.0.0.1:50051' service.KeyValue/WriteKeyValue
```

```
key-value-service_1    | OpenTelemetry trace error occurred. Exporter jaeger encountered the following error(s): thrift agent failed with not open
key-value-service_1    | OpenTelemetry trace error occurred. Exporter jaeger encountered the following error(s): thrift agent failed with not open
```

Related issues:

- https://github.com/open-telemetry/opentelemetry-rust/issues/851
- https://github.com/open-telemetry/opentelemetry-rust/issues/759

## References

See also:

- https://blessed.rs/crates
- https://tokio.rs/tokio/tutorial
- https://rust-lang.github.io/async-book/

Tracing:

- https://tokio.rs/tokio/topics/tracing-next-steps
- https://docs.rs/opentelemetry-jaeger/latest/opentelemetry_jaeger/
- https://docs.rs/opentelemetry/latest/opentelemetry/trace/trait.Tracer.html
- https://docs.rs/tracing/latest/tracing/span/index.html
- https://docs.rs/tonic/latest/tonic/transport/struct.Server.html#method.trace_fn
