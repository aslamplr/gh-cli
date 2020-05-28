# Stage 1 - build 
FROM rust:1.43-slim as build-deps

WORKDIR /usr/src/gh-cli

RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config 

COPY LICENSE .
COPY README.md .

COPY Cargo.* ./
COPY gh-lib/ gh-lib/
COPY gh-cli/ gh-cli/
COPY gh-web/ gh-web/

RUN cargo build --release -p gh-web

# Stage 2 - deploy 
FROM debian:buster-slim

LABEL maintainer="aslamplr@gmail.com"
LABEL version=1.0

WORKDIR /usr/src/gh-cli

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

COPY --from=build-deps /usr/src/gh-cli/target/release/gh-web /usr/local/bin/gh-web

ENV RUST_LOG=info
CMD ["gh-web"]

