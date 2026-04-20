# Base image with Rust toolchain (Debian-based with glibc)
# renovate: datasource=docker depName=rust versioning=semver
FROM rust:1.94.1@sha256:dbc91e219681fe9916c23882ca9b4b7b0485951c818a6781b02c889a30fd4e14

# Install clang, lld, and llvm for clang-cl backend (llvm provides llvm-lib)
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    lld \
    llvm \
    && rm -rf /var/lib/apt/lists/*

# Add Windows MSVC target and common Rust components
RUN rustup target add x86_64-pc-windows-msvc \
    && rustup component add rustfmt clippy

# Install cargo-xwin (musl binary)
# renovate: datasource=github-releases depName=rust-cross/cargo-xwin
ARG CARGO_XWIN_VERSION="0.21.5"
RUN wget -qO- "https://github.com/rust-cross/cargo-xwin/releases/download/v${CARGO_XWIN_VERSION}/cargo-xwin-v${CARGO_XWIN_VERSION}.x86_64-unknown-linux-musl.tar.gz" \
    | tar zxf - -C /usr/local/cargo/bin

# Install cargo-nextest
# renovate: datasource=crate depName=cargo-nextest
ARG NEXTEST_VERSION="0.9.122"
RUN wget -qO- "https://get.nexte.st/${NEXTEST_VERSION}/linux" | tar zxf - -C /usr/local/cargo/bin

# Install sccache for build caching (using musl binary)
# renovate: datasource=github-releases depName=mozilla/sccache
ARG SCCACHE_VERSION="0.14.0"
RUN wget -qO- "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar zxf - -C /tmp \
    && mv /tmp/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl/sccache /usr/local/cargo/bin/ \
    && rm -rf /tmp/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl

# Install just (using musl binary)
# renovate: datasource=github-releases depName=casey/just
ARG JUST_VERSION="1.50.0"
RUN wget -qO- "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar zxf - -C /usr/local/cargo/bin just

# Pre-cache Windows SDK and CRT
RUN cargo xwin cache xwin

# Set default target
ENV CARGO_BUILD_TARGET=x86_64-pc-windows-msvc
