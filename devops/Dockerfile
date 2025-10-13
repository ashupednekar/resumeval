FROM rust:1.86.0-bullseye AS build-env

WORKDIR /app

ARG GIT_TOKEN

RUN git config --global credential.helper store
RUN echo "https://ci:${GIT_TOKEN}@github.com" > ~/.git-credentials

COPY . /app

RUN cargo build --release -j 5

FROM gcr.io/distroless/cc-debian12

COPY --from=build-env /app/target/release/resumeval /app/resumeval
WORKDIR /app

CMD ["./resumeval", "listen"]
