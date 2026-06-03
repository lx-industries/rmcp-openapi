# Base image with Rust toolchain
FROM rust:1.96.0@sha256:fb328f0f58becb23ba1719940a2c94ece8b0b48afa837d05b79ef64bc1e18f6e

# Add common Rust components
RUN rustup component add rustfmt clippy

# Install cargo-nextest (faster test runner)
# renovate: datasource=github-releases depName=nextest-rs/nextest
ARG NEXTEST_VERSION="0.9.123"
RUN wget -qO- "https://get.nexte.st/${NEXTEST_VERSION}/linux" | tar zxf - -C /usr/local/cargo/bin

# Install typos-cli (spell checker)
# renovate: datasource=github-releases depName=crate-ci/typos
ARG TYPOS_VERSION="1.47.1"
RUN wget -qO- "https://github.com/crate-ci/typos/releases/download/v${TYPOS_VERSION}/typos-v${TYPOS_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar zxf - -C /usr/local/cargo/bin ./typos

# Install sccache for build caching
# renovate: datasource=github-releases depName=mozilla/sccache
ARG SCCACHE_VERSION="0.15.0"
RUN wget -qO- "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar zxf - -C /tmp \
    && mv /tmp/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl/sccache /usr/local/cargo/bin/ \
    && rm -rf /tmp/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl
