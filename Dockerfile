# One image for everything overstap runs on the NAS (deploy/nas/compose.yaml):
#   motis-run          MOTIS server, first import and nightly update (deploy/motis/run.sh)
#   overstap serve     web UI + API
#   overstap rt-log    GTFS-RT history logger

FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

# MOTIS ships as a static binary in its official image.
FROM ghcr.io/motis-project/motis:2.11.4 AS motis

FROM debian:bookworm-slim
LABEL org.opencontainers.image.title="overstap" \
      org.opencontainers.image.description="Personal multimodal journey planner for Belgium on MOTIS" \
      org.opencontainers.image.source="https://github.com/thepycoder/overstap"
# curl/python3: data downloads and the Hoppy feed; pyosmium: cleaning a non-Geofabrik OSM
# extract (ADR-009); util-linux: flock; tzdata: the nightly update runs on local time.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl python3 python3-pyosmium tzdata util-linux \
 && rm -rf /var/lib/apt/lists/*
COPY --from=motis /motis /usr/local/bin/motis
COPY --from=motis /ui /app/motis-ui
COPY --from=build /src/target/release/overstap /usr/local/bin/overstap
COPY web /app/web
COPY tools /app/tools
COPY deploy/motis/config.yml deploy/motis/update.sh deploy/motis/run.sh /app/motis/
COPY deploy/nas/motis-run /usr/local/bin/motis-run
ENV TZ=Europe/Brussels \
    MOTIS_DIR=/data/motis \
    MOTIS_BIN=/usr/local/bin/motis \
    MOTIS_URL=http://localhost:8080 \
    OVERSTAP_WEB=/app/web
EXPOSE 8080 8737
CMD ["overstap", "serve", "--listen", "0.0.0.0:8737"]
