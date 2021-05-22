FROM rust:1.52 as base
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools
RUN rustup target add x86_64-unknown-linux-musl

FROM base as planner
# We only pay the installation cost once, 
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning 
# the cargo-chef version with `--version X.X.X`
RUN cargo install cargo-chef --version 0.1.20
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM base as cacher
RUN cargo install cargo-chef --version 0.1.20
COPY --from=planner /app/recipe.json recipe.json
# # add following 2 lines after initial build to speed up next builds
# COPY --from=jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher /app/target target
# COPY --from=jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher /usr/local/cargo /usr/local/cargo
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

FROM base
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
RUN cargo test --release --target x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
USER 1000
COPY --from=alpine:latest /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=3 /app/target/x86_64-unknown-linux-musl/release/jarvis-alpha-innotec-exporter .
ENTRYPOINT ["./jarvis-alpha-innotec-exporter"]