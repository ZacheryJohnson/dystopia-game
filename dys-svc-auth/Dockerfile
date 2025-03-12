# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.85-bookworm AS builder

ARG WORKING_DIR="."

# Bevy has special requirements on Linux
RUN apt update
RUN apt install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev protobuf-compiler

COPY $WORKING_DIR/Cargo.toml /opt/dystopia/Cargo.toml
COPY $WORKING_DIR/dys-datastore/ /opt/dystopia/dys-datastore/
COPY $WORKING_DIR/dys-datastore-valkey/ /opt/dystopia/dys-datastore-valkey/
COPY $WORKING_DIR/dys-nats /opt/dystopia/dys-nats
COPY $WORKING_DIR/dys-protocol /opt/dystopia/dys-protocol
COPY $WORKING_DIR/dys-observability/ /opt/dystopia/dys-observability/
COPY $WORKING_DIR/dys-svc-auth/ /opt/dystopia/dys-svc-auth/

WORKDIR /opt/dystopia/dys-svc-auth
RUN cargo build --release -p dys-svc-auth

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim AS runtime

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-auth /dys-svc-auth

CMD [ "/dys-svc-auth" ]