FROM ubuntu:16.04

RUN apt-get update
RUN apt-get install -y --no-install-recommends \
      gcc libc6-dev curl file ca-certificates
COPY bin/compile.sh bin/evaluate.sh /usr/local/bin/
COPY install.sh /tmp/
RUN sh /tmp/install.sh stable
USER nobody

WORKDIR /tmp
