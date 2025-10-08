FROM rust:latest

WORKDIR /blindserver
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release

CMD ["cargo", "run", "--release"]
