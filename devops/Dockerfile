FROM rust:1.86.0-bullseye AS build-env
WORKDIR /app
COPY . /app
RUN cargo build --release -j 5
RUN cp /app/target/release/resumeval /app/resumeval
#FROM debian:12-slim
#RUN apt-get update && apt-get install -y \
#    ca-certificates \
#    libssl3 \
#    && rm -rf /var/lib/apt/lists/*
#COPY --from=build-env /app/target/release/resumeval /app/resumeval
#WORKDIR /app
CMD ["./resumeval", "listen"]
