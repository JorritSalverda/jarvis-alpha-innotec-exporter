FROM rust:1.52 as builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
# add following 2 lines after initial build to speed up next builds
COPY --from=jsalverda/jarvis-alpha-innotec-exporter:dlc-builder /app/target target
COPY --from=jsalverda/jarvis-alpha-innotec-exporter:dlc-builder /usr/local/cargo /usr/local/cargo
RUN cargo test --release --target x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch AS runtime
USER 1000
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/jarvis-alpha-innotec-exporter .
ENTRYPOINT ["./jarvis-alpha-innotec-exporter"]