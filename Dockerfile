FROM rust:1.42.0 AS builder
RUN apt update && apt install libssl-dev -y

WORKDIR /build
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new hello-boys
WORKDIR /build/hello-boys
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch

WORKDIR /hello-boys
COPY --from=builder /usr/local/cargo/bin/hello-boys .

CMD [ "./hello-boys" ] 
