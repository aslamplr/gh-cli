FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.0

COPY openssl.sh /
RUN bash /openssl.sh linux-armv4 arm-linux-gnueabihf- 

ENV OPENSSL_DIR=/openssl \
    OPENSSL_INCLUDE_DIR=/openssl/include \
    OPENSSL_LIB_DIR=/openssl/lib

# curl and libz deps are for build time only 
RUN apt-get update && \
    apt-get install --assume-yes curl pkg-config libz-dev 

