# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.79-bookworm AS builder

ARG WORKING_DIR="."

# Bevy has special requirements on Linux
RUN apt update
RUN apt install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev

# Install nodejs, which includes npm
RUN apt install -y nodejs npm

# Add WASM dependencies
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli
RUN cargo install wasm-opt --locked

WORKDIR /opt/dystopia
COPY $WORKING_DIR/Cargo.toml /opt/dystopia/Cargo.toml
COPY $WORKING_DIR/dys-game/ /opt/dystopia/dys-game/
COPY $WORKING_DIR/dys-world/ /opt/dystopia/dys-world/

# Building the match visualizer is the longest part of this process
# Ensure it's as high up in the stage as possible to prevent subsequent changes from invaliding this if cached 
COPY $WORKING_DIR/dys-matchvisualizer/ /opt/dystopia/dys-matchvisualizer/
RUN dys-matchvisualizer/build_scripts/matchvisualizer-wasm.sh

COPY $WORKING_DIR/dys-observability/ /opt/dystopia/dys-observability/
COPY $WORKING_DIR/dys-svc-webapp/ /opt/dystopia/dys-svc-webapp/
RUN dys-svc-webapp/build_scripts/build_webapp_frontend.sh dys-svc-webapp

WORKDIR /opt/dystopia/dys-svc-webapp
RUN cargo build --release -p dys-svc-webapp

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim AS runtime

EXPOSE 6080/tcp

RUN apt update
RUN apt install libssl3

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-webapp /dys-svc-webapp

# Copy static files to be served
# ZJ-TODO: this should be an attached volume instead
#          we don't need every container with the static files, but instead one shared source of truth
ENV DIST_PATH /dist
COPY --from=builder /opt/dystopia/dys-svc-webapp/frontend/dist /dist/

CMD [ "/dys-svc-webapp" ]
