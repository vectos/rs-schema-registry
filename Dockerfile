FROM rust:1.69.0 as builder
WORKDIR /usr/src/rs-schema-registry
COPY . .
RUN cargo install --path .
FROM debian:buster-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rs-schema-registry /usr/local/bin/rs-schema-registry
CMD ["rs-schema-registry"]