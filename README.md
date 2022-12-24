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
$ grpcurl -plaintext -import-path ./proto -proto service.proto -d '{"keys": ["Key"]}' '[::]:50051' service.KeyValue/ReadKeyValue
```

```
grpcurl -plaintext -import-path ./proto -proto service.proto -d '{"items": [{"key": "foo", "value": "bar"}]}' '[::]:50051' service.KeyValue/WriteKeyValue
```

## References

See also:

- https://blessed.rs/crates
- https://tokio.rs/tokio/tutorial
- https://rust-lang.github.io/async-book/
