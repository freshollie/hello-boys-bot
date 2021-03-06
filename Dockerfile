FROM clux/muslrust:1.45.2-stable AS builder

WORKDIR /build

RUN USER=root cargo new hello-boys
WORKDIR /build/hello-boys

COPY Cargo.toml Cargo.lock ./
COPY assets assets
COPY src src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM alpine:3.12.0

WORKDIR /hello-boys
COPY --from=builder /root/.cargo/bin/hello-boys .

CMD [ "/hello-boys/hello-boys" ] 
