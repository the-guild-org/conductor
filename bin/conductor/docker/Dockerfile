FROM rust:1.76.0 as builder

WORKDIR /usr/src/conductor

COPY Cargo.toml Cargo.lock ./
# This is a trick to get the most out of Docker's caching mechanism in GH Actions.
COPY libs libs
COPY plugins plugins
COPY bin bin
RUN rm -rf lib/benches
RUN rm -rf lib/e2e_tests
RUN rm -rf lib/smoke_tests
RUN echo 'fn main() { println!(""); }' > ./bin/conductor/src/main.rs
RUN echo 'fn main() { println!(""); }' > ./bin/conductor/src/lib.rs
# We are only building the dependencies here, with a dummy file, this compiles all dependencies code only.
RUN cargo build --release --bin conductor

# Now we can remove the dummy code, copy the actual code and compile the user code.
# This ensures that building dependencies and the actual code are cached separately.
COPY bin/conductor/src/main.rs bin/conductor/src/main.rs
COPY bin/conductor/src/lib.rs bin/conductor/src/lib.rs
RUN touch bin/conductor/src/main.rs bin/conductor/src/lib.rs
RUN cargo build --release --bin conductor

FROM debian:12.5

RUN apt-get update -y && apt-get install -y ca-certificates

COPY --from=builder /usr/src/conductor/target/release/conductor /usr/local/bin/conductor

CMD conductor
