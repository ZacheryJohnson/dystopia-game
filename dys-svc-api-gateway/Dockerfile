# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.90-bookworm AS builder

ARG WORKING_DIR="."

RUN apt-get update
RUN apt-get install -y g++ pkg-config protobuf-compiler

COPY $WORKING_DIR/.cargo/config.toml /opt/dystopia/.cargo/config.toml
COPY $WORKING_DIR/Cargo.toml /opt/dystopia/Cargo.toml
COPY $WORKING_DIR/dys-*/.. /opt/dystopia/

WORKDIR /opt/dystopia/dys-svc-api-gateway
RUN cargo run --release -p dys-svc-director --bin gen-openapi
RUN cargo build --release -p dys-svc-api-gateway --verbose

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim AS runtime
EXPOSE 6050/tcp

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-api-gateway /dys-svc-api-gateway

# Copy the generated openapi file
COPY --from=builder /opt/dystopia/dys-svc-api-gateway/generated/openapi.json /openapi.json

ENV OPENAPI_SPEC_PATH=/openapi.json

CMD [ "/dys-svc-api-gateway" ]