FROM rust:1.90 AS builder
WORKDIR /shoclips
COPY . .

RUN apt-get update && apt-get install -y \
    cmake \
    protobuf-compiler \
    clang \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/shoclips /usr/local/bin/shoclips
CMD ["shoclips"]