# Change
FROM devkitpro/devkitarm:20250527 AS devkitpro

FROM rust:trixie

COPY --from=devkitpro /opt/devkitpro /opt/devkitpro

ENV DEVKITPRO=/opt/devkitpro
ENV DEVKITARM=${DEVKITPRO}/devkitARM
ENV PATH=${DEVKITPRO}/tools/bin:$PATH
ENV PATH=${DEVKITARM}/bin:$PATH

ENV RUST_TOOLCHAIN_SDOP_GAME=nightly-2025-07-23

ENV VITASDK=/usr/local/vitasdk
ENV PATH=$VITASDK/bin:$PATH

RUN apt-get update && \
    apt-get install --no-install-recommends -y make zip gcc g++ gcc-arm-none-eabi clang libsdl2-image-dev libsdl2-dev cmake nodejs git curl sudo wget bzip2 xz-utils && \
    cargo install agb-gbafix && \
    cargo install --locked trunk && \
    cargo install --locked cargo-3ds && \
    cargo install cargo-psp && \
    cargo +nightly install cargo-vita && \
    rustup target add x86_64-unknown-linux-gnu --toolchain ${RUST_TOOLCHAIN_SDOP_GAME} && \
    rustup target add wasm32-unknown-unknown --toolchain ${RUST_TOOLCHAIN_SDOP_GAME} && \
    rustup target add thumbv8m.main-none-eabihf --toolchain ${RUST_TOOLCHAIN_SDOP_GAME} && \
    rustup component add rust-src --toolchain ${RUST_TOOLCHAIN_SDOP_GAME}-x86_64-unknown-linux-gnu && \
    rustup component add rust-src --toolchain ${RUST_TOOLCHAIN_SDOP_GAME} && \
    rustup component add clippy --toolchain ${RUST_TOOLCHAIN_SDOP_GAME} && \
    rustup component add rustfmt --toolchain ${RUST_TOOLCHAIN_SDOP_GAME} && \
    cargo install cargo-cache && \
    cargo cache -a && \
    git clone https://github.com/vitasdk/vdpm && cd vdpm && ./bootstrap-vitasdk.sh && ./install-all.sh  && cd .. && \
    useradd -ms /bin/bash builder && \
    mkdir /app && \
    chown -R builder:builder /app && \
    chown -R builder:builder /usr/local/cargo && \
    chown -R builder:builder ${DEVKITPRO}

USER builder

WORKDIR /app
