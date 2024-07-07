# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.79-bookworm as builder
# Bevy has special requirements on Linux
RUN apt-get update
RUN apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev

# Install nodejs, which includes npm
RUN apt-get install -y nodejs

# Add WASM dependencies
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli
RUN cargo install wasm-opt --locked

COPY ./Cargo.toml /opt/dystopia/Cargo.toml
COPY ./dys-game/ /opt/dystopia/dys-game/
COPY ./dys-world/ /opt/dystopia/dys-world/
COPY ./dys-matchvisualizer/ /opt/dystopia/dys-matchvisualizer/
COPY ./dys-svc-webapp/ /opt/dystopia/dys-svc-webapp/

WORKDIR /opt/dystopia
RUN dys-matchvisualizer/build_scripts/matchvisualizer-wasm.sh

WORKDIR /opt/dystopia/dys-svc-webapp
RUN cargo build --release -p dys-svc-webapp
RUN ./build_scripts/build_webapp_frontend.sh

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim as runtime
EXPOSE 6080/tcp

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-webapp /dys-svc-webapp

# Copy static files to be served
# ZJ-TODO: this should be an attached volume instead
#          we don't need every container with the static files, but instead one shared source of truth
ENV DIST_PATH /dist
COPY --from=builder /opt/dystopia/dys-svc-webapp/frontend/dist /dist/

CMD [ "/dys-svc-webapp" ]