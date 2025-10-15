# syntax=docker/dockerfile:1
FROM rust:bullseye AS build

ARG GITLAB_TOKEN

WORKDIR /usr/src/dolphin

# Change APT REGISTRY
RUN sed -i 's|deb.debian.org/debian|ftp.kaist.ac.kr/debian|' /etc/apt/sources.list && \
    sed -i 's|security.debian.org/debian-security|ftp.kaist.ac.kr/debian-security|' /etc/apt/sources.list

RUN apt-get update && \
    apt-get install -y libmariadb-dev libssl-dev build-essential cmake git

#RUN git clone https://sxxphia:${GITLAB_TOKEN}@git.stl1.co.kr/stl/bizppurio-rs.git /usr/src/bizppurio-rs
RUN git config --global https.sslVerify false && \
    git clone https://github.com/wacker23/bizppurio-rs.git /usr/src/bizppurio-rs

COPY . .

RUN cargo build -r -j`(getconf _NPROCESSORS_ONLN)`

FROM debian:bullseye-slim

# Change APT REGISTRY
RUN sed -i 's|deb.debian.org/debian|ftp.kaist.ac.kr/debian|' /etc/apt/sources.list && \
    sed -i 's|security.debian.org/debian-security|ftp.kaist.ac.kr/debian-security|' /etc/apt/sources.list

RUN apt-get update && \
    apt-get install -y libmariadb-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

RUN addgroup --system --gid 1001 admin
RUN adduser --system --uid 1001  stl

COPY --from=build --chown=stl:admin /usr/src/dolphin/target/release/dolphin /usr/sbin

RUN /usr/sbin/dolphin -v > /dev/stderr

CMD ["dolphin"]
