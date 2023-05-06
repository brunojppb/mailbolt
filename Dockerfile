FROM rust:1.68.2 as builder

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM debian:buster-slim as runtime

WORKDIR "/app"
# Make sure the app does not run as root
RUN chown nobody /app

# OpenSSL is dynamically linked by some of our dependencies.
# ca-certificates - it is needed to verify TLS certificates when establishing HTTPS connections
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

# Only copy the final release from the build stage
COPY --from=builder --chown=nobody:root /app/target/release/mailbolt ./
# Make sure the config files are available
COPY --from=builder --chown=nobody:root /app/config ./config

USER nobody

# Instruct the config setup to read production values
ENV APP_ENV prod
ENTRYPOINT ["./mailbolt"]