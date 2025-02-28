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
COPY $WORKING_DIR/dys-protocol /opt/dystopia/dys-protocol
COPY $WORKING_DIR/dys-satisfiable/ /opt/dystopia/dys-satisfiable/
COPY $WORKING_DIR/dys-satisfiable-macros/ /opt/dystopia/dys-satisfiable-macros/
COPY $WORKING_DIR/dys-simulation/ /opt/dystopia/dys-simulation/
COPY $WORKING_DIR/dys-observability/ /opt/dystopia/dys-observability/
COPY $WORKING_DIR/dys-world/ /opt/dystopia/dys-world/
COPY $WORKING_DIR/dys-svc-director/ /opt/dystopia/dys-svc-director/

WORKDIR /opt/dystopia/dys-svc-director
RUN cargo build --release -p dys-svc-director

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim AS runtime

EXPOSE 6081/tcp

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-director /dys-svc-director

CMD [ "/dys-svc-director" ]