FROM docker.io/debian:trixie-slim

RUN apt-get update && \
    apt-get install -y procps less git rustup build-essential vim-tiny && \
    apt-get clean

ENV PATH=${PATH}:/root/.local/bin
