FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.0

# curl and libz deps are for build time only
RUN apt-get update && \
    apt-get install --assume-yes libssl-dev curl pkg-config libz-dev
