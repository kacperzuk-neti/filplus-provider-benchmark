# syntax = docker/dockerfile:1.2
FROM lukemathwalker/cargo-chef:latest-rust-1.80.1-slim-bullseye as base
RUN apt-get update && apt-get -y install clang cmake perl libfindbin-libs-perl
WORKDIR /app

FROM base as plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base as build
ARG GITHUB_SHA
ENV GITHUB_SHA ${GITHUB_SHA}
COPY --from=plan /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bins

FROM debian:bullseye-slim as run
RUN apt-get update && apt-get -y install ca-certificates libc6

COPY --from=build /app/target/release/scheduler /usr/local/bin/
COPY --from=build /app/target/release/worker /usr/local/bin/

RUN adduser --system --group --no-create-home bmsuser
USER bmsuser