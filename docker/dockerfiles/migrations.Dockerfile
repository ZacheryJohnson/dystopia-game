FROM rust:1.89-bookworm
VOLUME /migrations
RUN cargo install sqlx-cli
COPY docker/dockerfiles/migrate.sh /scripts/migrate.sh

CMD ["/scripts/migrate.sh", "/migrations"]
