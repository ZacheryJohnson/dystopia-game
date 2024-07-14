# --------------------------------------------
#   Build
# --------------------------------------------
FROM rust:1.79-bookworm as builder
# Bevy has special requirements on Linux
RUN apt update
RUN apt install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev

COPY ./Cargo.toml /opt/dystopia/Cargo.toml
COPY ./dys-svc-director/ /opt/dystopia/dys-svc-director/
COPY ./dys-game/ /opt/dystopia/dys-game/
COPY ./dys-world/ /opt/dystopia/dys-world/

WORKDIR /opt/dystopia/dys-svc-director
RUN cargo build --release -p dys-svc-director

# --------------------------------------------
#   Runtime
# --------------------------------------------
FROM debian:bookworm-slim as runtime
EXPOSE 6081/tcp

# Copy the binary
COPY --from=builder /opt/dystopia/target/release/dys-svc-director /dys-svc-director

CMD [ "/dys-svc-director" ]