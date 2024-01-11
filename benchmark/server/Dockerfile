FROM rust:1.75.0 as builder

WORKDIR /usr/src/conductor-source

COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/actix_hello_server /usr/local/bin/actix_hello_server
CMD ["server"]
