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

# Only copy the final release from the build stage
COPY --from=builder --chown=nobody:root /app/target/release/mailbolt ./
# Make sure the config files are available
COPY --from=builder --chown=nobody:root /app/config ./config

USER nobody

# Instruct the config setup to read production values
ENV APP_ENV prod
ENTRYPOINT ["./mailbolt"]