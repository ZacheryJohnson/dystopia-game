# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.85-bookworm AS builder

ARG WORKING_DIR="."
ARG RUST_PROJECT_NAME="dys-matchvisualizer"
ARG OUT_DIR="wasm_out"
ARG BIN_NAME="matchvisualizer"
ARG WEBAPP_PUBLIC_DIR_PATH="dys-svc-webapp/frontend/public"
ARG WEBAPP_INTERNAL_DIR_PATH="dys-svc-webapp/frontend/src/assets"

# Bevy has special requirements on Linux
RUN apt update
RUN apt install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev protobuf-compiler

# Install nodejs, which includes npm
RUN apt install -y nodejs npm

# Add WASM dependencies
RUN rustup target add wasm32-unknown-unknown
RUN cargo install -f wasm-bindgen-cli --version 0.2.100
RUN cargo install -f wasm-opt

WORKDIR /opt/dystopia
COPY $WORKING_DIR/Cargo.toml /opt/dystopia/Cargo.toml
COPY $WORKING_DIR/dys-datastore/ /opt/dystopia/dys-datastore/
COPY $WORKING_DIR/dys-datastore-valkey/ /opt/dystopia/dys-datastore-valkey/
COPY $WORKING_DIR/dys-nats /opt/dystopia/dys-nats
COPY $WORKING_DIR/dys-protocol /opt/dystopia/dys-protocol
COPY $WORKING_DIR/dys-satisfiable/ /opt/dystopia/dys-satisfiable/
COPY $WORKING_DIR/dys-satisfiable-macros/ /opt/dystopia/dys-satisfiable-macros/
COPY $WORKING_DIR/dys-simulation/ /opt/dystopia/dys-simulation/
COPY $WORKING_DIR/dys-world/ /opt/dystopia/dys-world/
COPY $WORKING_DIR/dys-matchvisualizer/ /opt/dystopia/dys-matchvisualizer/

RUN cargo build -p $RUST_PROJECT_NAME --release --target wasm32-unknown-unknown

RUN mkdir $OUT_DIR
RUN wasm-bindgen --target web \
    --out-dir $OUT_DIR \
    --out-name $BIN_NAME \
    target/wasm32-unknown-unknown/release/$RUST_PROJECT_NAME.wasm

RUN wasm-opt -Oz -o $OUT_DIR/${BIN_NAME}_opt.wasm $OUT_DIR/${BIN_NAME}_bg.wasm

COPY $WORKING_DIR/dys-observability/ /opt/dystopia/dys-observability/
COPY $WORKING_DIR/dys-svc-webapp/ /opt/dystopia/dys-svc-webapp/
RUN cp $OUT_DIR/matchvisualizer_opt.wasm /opt/dystopia/$WEBAPP_PUBLIC_DIR_PATH/matchvisualizer_opt.wasm
RUN cp $OUT_DIR/matchvisualizer.d.ts /opt/dystopia/$WEBAPP_INTERNAL_DIR_PATH/matchvisualizer.d.ts
RUN cp $OUT_DIR/matchvisualizer.js /opt/dystopia/$WEBAPP_INTERNAL_DIR_PATH/matchvisualizer.js

WORKDIR /opt/dystopia/dys-svc-webapp/frontend
RUN npm install
RUN npm run build

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim AS runtime
EXPOSE 5173/tcp
EXPOSE 5174/tcp

RUN apt update
RUN apt install -y nodejs npm

COPY dys-svc-webapp/frontend /frontend/

WORKDIR /frontend/
RUN npm install
CMD [ "npm", "run", "dev" ]
