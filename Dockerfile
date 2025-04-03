FROM ubuntu:jammy AS builder

ARG PROFILE=release

RUN apt-get update \
    && apt-get -y install build-essential curl libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN /root/.cargo/bin/rustup update

COPY . /app
WORKDIR /app
RUN /root/.cargo/bin/cargo build --$PROFILE --package crunch

# ===== SECOND STAGE ======
FROM ubuntu:jammy

RUN apt-get update \
    && apt-get -y install ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG PROFILE=release
COPY --from=builder /app/target/$PROFILE/crunch /usr/local/bin

RUN useradd -u 1000 -U -s /bin/sh crunch
USER crunch

ENV RUST_BACKTRACE=1
ENV RUST_LOG="info"

RUN /usr/local/bin/crunch --version

ENTRYPOINT [ "/usr/local/bin/crunch" ]
