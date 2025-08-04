# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.88-bookworm AS builder

ARG WORKING_DIR="."

# Bevy has special requirements on Linux
RUN apt-get update
RUN apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev protobuf-compiler

COPY $WORKING_DIR/.cargo/config.toml /opt/dystopia/.cargo/config.toml
COPY $WORKING_DIR/.sqlx /opt/dystopia/.sqlx
COPY $WORKING_DIR/Cargo.toml /opt/dystopia/Cargo.toml
COPY $WORKING_DIR/dys-datastore/ /opt/dystopia/dys-datastore/
COPY $WORKING_DIR/dys-datastore-mysql/ /opt/dystopia/dys-datastore-mysql/
COPY $WORKING_DIR/dys-datastore-valkey/ /opt/dystopia/dys-datastore-valkey/
COPY $WORKING_DIR/dys-nats /opt/dystopia/dys-nats
COPY $WORKING_DIR/dys-protocol /opt/dystopia/dys-protocol
COPY $WORKING_DIR/dys-satisfiable/ /opt/dystopia/dys-satisfiable/
COPY $WORKING_DIR/dys-satisfiable-macros/ /opt/dystopia/dys-satisfiable-macros/
COPY $WORKING_DIR/dys-simulation/ /opt/dystopia/dys-simulation/
COPY $WORKING_DIR/dys-stat/ /opt/dystopia/dys-stat/
COPY $WORKING_DIR/dys-observability/ /opt/dystopia/dys-observability/
COPY $WORKING_DIR/dys-world/ /opt/dystopia/dys-world/
COPY $WORKING_DIR/dys-svc-director/ /opt/dystopia/dys-svc-director/
COPY $WORKING_DIR/dys-svc-director/migrations /opt/dystopia/migrations

ENV SQLX_OFFLINE=true

WORKDIR /opt/dystopia/dys-svc-director
RUN cargo build --release -p dys-svc-director

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim AS runtime

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-director /dys-svc-director
COPY --from=builder /opt/dystopia/migrations /migrations

ENV MINUTES_BETWEEN_MATCHES=3

CMD [ "/dys-svc-director" ]