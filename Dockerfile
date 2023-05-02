FROM rust:1.68.2 as builder

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:1.68.2 as runtime

WORKDIR "/app"
# Make sure the app does not run as root
RUN chown nobody /app

# Only copy the final release from the build stage
COPY --from=builder --chown=nobody:root /app/target/release/mailbolt ./
COPY --from=builder --chown=nobody:root /app/configuration.yaml ./

USER nobody

ENTRYPOINT ["./mailbolt"]