FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

# Install basic dependencies
RUN apt-get update && apt-get install -y \
    wget \
    ca-certificates \
    xdotool \
    && rm -rf /var/lib/apt/lists/*

# Copy .deb package (will be mounted at runtime)
WORKDIR /test

CMD ["/bin/bash"]
