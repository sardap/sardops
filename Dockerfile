# Change
FROM devkitpro/devkitarm:20250527 AS devkitpro

FROM rust:1.88.0-slim-bullseye

COPY --from=devkitpro /opt/devkitpro /opt/devkitpro

ENV DEVKITPRO=/opt/devkitpro
ENV DEVKITARM=${DEVKITPRO}/devkitARM
ENV PATH=${DEVKITPRO}/tools/bin:$PATH
ENV PATH=${DEVKITARM}/bin:$PATH

ENV RUST_TOOLCHAIN=nightly-2025-09-08

RUN apt-get update && \
    apt-get install --no-install-recommends -y make zip gcc g++ gcc-arm-none-eabi clang libsdl2-image-dev libsdl2-dev cmake && \
    cargo install agb-gbafix && \
    cargo install trunk && \
    cargo install --locked cargo-3ds && \
    cargo install cargo-psp && \
    rustup target add x86_64-unknown-linux-gnu --toolchain ${RUST_TOOLCHAIN} && \
    rustup target add wasm32-unknown-unknown --toolchain ${RUST_TOOLCHAIN} && \
    rustup target add thumbv8m.main-none-eabihf --toolchain ${RUST_TOOLCHAIN} && \
    rustup component add rust-src --toolchain ${RUST_TOOLCHAIN} && \
    rustup component add clippy --toolchain ${RUST_TOOLCHAIN} && \
    rustup component add rustfmt --toolchain ${RUST_TOOLCHAIN} && \
    cargo install cargo-cache && \
    cargo cache -a

RUN useradd -ms /bin/bash builder
USER builder

USER root
RUN mkdir /app
RUN chown -R builder:builder /app
RUN chown -R builder:builder /usr/local/cargo
RUN chown -R builder:builder ${DEVKITPRO}

RUN apt-get install -y nodejs
USER builder

USER root

WORKDIR /app
