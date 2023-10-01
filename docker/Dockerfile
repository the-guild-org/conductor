FROM rust:1.72.1 as builder

WORKDIR /usr/src/conductor

COPY Cargo.toml Cargo.lock ./
# This is a trick to get the most out of Docker's caching mechanism in GH Actions.
RUN mkdir src
RUN mkdir benches
RUN echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN echo 'fn main() { println!("Dummy!"); }' > ./src/lib.rs
RUN echo 'fn main() { println!("Dummy!"); }' > ./benches/bench.rs
# We are only building the dependencies here, with a dummy file, this compiles all dependencies code only.
RUN cargo build --release --bin conductor

# Now we can remove the dummy code, copy the actual code and compile the user code.
# This ensures that building dependencies and the actual code are cached separately.
RUN rm -rf src
COPY src src
RUN touch src/main.rs src/lib.rs
RUN cargo build --release --bin conductor

FROM debian:12.1

RUN apt-get update -y && apt-get install -y ca-certificates

COPY --from=builder /usr/src/conductor/target/release/conductor /usr/local/bin/conductor

CMD conductor
