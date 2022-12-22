FROM rust:1.64-buster

RUN apt update -q
RUN apt install -y -qq python3-protobuf protobuf-compiler

RUN mkdir -p /cache/cargo-target /cache/cargo-home

ENV CARGO_TARGET_DIR /cache/cargo-target
ENV CARGO_HOME       /cache/cargo-home

COPY . /code/
WORKDIR /code/
RUN --mount=type=cache,target=/cache/cargo-target,sharing=locked \
    cargo build && cargo install --path .

ENV PATH="$PATH:/cache/cargo-home/bin/"
