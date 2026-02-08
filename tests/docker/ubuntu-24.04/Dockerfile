FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    wget \
    ca-certificates \
    xdotool \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /test

CMD ["/bin/bash"]
