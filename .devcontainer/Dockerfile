# Dev container for rmcp-openapi
# Extends the CI image with development tools (Claude Code, forge CLIs, Docker
# CLI, SSH, firewall).
#
# No WORKDIR — project mounted at host-native absolute path
# for git worktree compatibility (.git file contains absolute paths).

# -- Base: CI image (Rust toolchain + cargo-nextest, typos, sccache) -----------
# renovate: datasource=docker registryUrl=https://registry.gitlab.com depName=lx-industries/rmcp-openapi/images/rust
FROM registry.gitlab.com/lx-industries/rmcp-openapi/images/rust:1.96.1-x86_64-unknown-linux-gnu@sha256:afed6a61db49db335dd876ce95fc820b221be82d81f31cab6275830610aca019

# -- Environment (set early so tool installers use the right paths) -----------
ENV HOME=/tmp/home
ENV PATH="${HOME}/.local/bin:${PATH}"

# -- NPM supply-chain hardening ------------------------------------------------
# Block postinstall scripts and require 24h package age for supply-chain safety.
ENV NPM_CONFIG_IGNORE_SCRIPTS=true
ENV NPM_CONFIG_MINIMUM_RELEASE_AGE=1440

# -- Forge CLI config directories (writable for non-root users) ----------------
ENV GLAB_CONFIG_DIR=/tmp/glab-config
ENV GH_CONFIG_DIR=/tmp/gh-config

# -- System packages -----------------------------------------------------------
RUN apt-get update -qq \
    && apt-get install -y -qq --no-install-recommends \
        openssh-client \
        ca-certificates gnupg \
        iptables ipset iproute2 dnsutils curl \
    && rm -rf /var/lib/apt/lists/*

# -- Docker CLI + Compose plugin (host daemon via socket mount; no daemon here) -
RUN <<'INSTALL_DOCKER'
  set -eux
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/debian/gpg \
    | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  chmod a+r /etc/apt/keyrings/docker.gpg
  echo "deb [arch=$(dpkg --print-architecture) \
    signed-by=/etc/apt/keyrings/docker.gpg] \
    https://download.docker.com/linux/debian \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update
  apt-get install -y --no-install-recommends \
    docker-ce-cli docker-compose-plugin docker-buildx-plugin
  rm -rf /var/lib/apt/lists/*
INSTALL_DOCKER

# -- gosu (privilege drop for firewalled mode + IDE compatibility) -------------
# renovate: datasource=github-releases depName=tianon/gosu
ARG GOSU_VERSION="1.19"
RUN arch="$(dpkg --print-architecture)" \
    && curl -fsSL "https://github.com/tianon/gosu/releases/download/${GOSU_VERSION}/gosu-${arch}" -o /usr/local/bin/gosu \
    && chmod +x /usr/local/bin/gosu \
    && gosu --version

# -- Node.js (for integration tests — test_with_js) ---------------------------
# renovate: datasource=node-version depName=node
ARG NODE_VERSION="24.20.0"
RUN curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz" \
    | tar -xJ -C /usr/local --strip-components=1

# -- Claude Code (native binary, auto-updates) --------------------------------
# HOME is set above so the installer places the binary at $HOME/.local/bin/claude
RUN curl -fsSL https://claude.ai/install.sh | bash \
    && claude --version

# -- Login-shell PATH restore (userEnvProbe: loginInteractiveShell) -----------
# A login shell sources /etc/profile, which OVERWRITES PATH with the distro
# default instead of extending it — dropping everything ENV PATH added above
# ($HOME/.local/bin for Claude Code, /usr/local/cargo/bin from the base image).
# That is the environment the IDE hands to rust-analyzer / LSP / the AI tools.
# Adopts the devcontainers/features idiom verbatim:
# https://github.com/devcontainers/features/blob/main/src/rust/install.sh#L99-L102
#
# Must run under bash — ${PATH//...} is a bashism, and Docker's default RUN
# shell is /bin/sh (dash), which fails this with exit code 2.
# Placed AFTER every ENV PATH line it is meant to capture.
RUN bash -c ' \
    rm -f /etc/profile.d/00-restore-env.sh && \
    echo "export PATH=${PATH//$(sh -lc '"'"'echo $PATH'"'"')/\$PATH}" > /etc/profile.d/00-restore-env.sh && \
    chmod +x /etc/profile.d/00-restore-env.sh \
'

# -- GitLab CLI (glab) ---------------------------------------------------------
# renovate: datasource=gitlab-releases depName=gitlab-org/cli registryUrl=https://gitlab.com
ARG GLAB_VERSION="1.115.0"
RUN arch="$(dpkg --print-architecture)" \
    && curl -fsSL "https://gitlab.com/gitlab-org/cli/-/releases/v${GLAB_VERSION}/downloads/glab_${GLAB_VERSION}_linux_${arch}.deb" -o /tmp/glab.deb \
    && dpkg -i /tmp/glab.deb \
    && rm /tmp/glab.deb \
    && glab --version \
    && rm -rf /tmp/glab-config \
    && mkdir -m 1777 /tmp/glab-config

# -- GitHub CLI (gh) -----------------------------------------------------------
# renovate: datasource=github-releases depName=cli/cli
ARG GH_VERSION="2.99.0"
RUN arch="$(dpkg --print-architecture)" \
    && curl -fsSL "https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_${arch}.deb" -o /tmp/gh.deb \
    && dpkg -i /tmp/gh.deb \
    && rm /tmp/gh.deb \
    && gh --version \
    && rm -rf /tmp/gh-config \
    && mkdir -m 1777 /tmp/gh-config

# -- Git and SSH configuration -------------------------------------------------
RUN git config --system safe.directory '*'
# -T 15: the default 5s timeout can silently yield no key on a slow banner.
# `|| true`: a transient network miss must not break the build.
RUN mkdir -p /etc/ssh \
    && { ssh-keyscan -T 15 -t ecdsa,rsa,ed25519 gitlab.com github.com >> /etc/ssh/ssh_known_hosts 2>/dev/null || true; }

# -- Firewall assets -----------------------------------------------------------
COPY firewall.sh /usr/local/bin/firewall.sh
RUN chmod +x /usr/local/bin/firewall.sh

COPY firewall-allowlist.txt /usr/local/share/firewall-allowlist.txt

# firewall-list — inspect allowed IPs (reads snapshot taken before privilege drop)
RUN printf '#!/bin/bash\ncat /tmp/firewall-allowed-ips.txt 2>/dev/null || echo "Firewall not active"\n' \
        > /usr/local/bin/firewall-list \
    && chmod +x /usr/local/bin/firewall-list

# -- Non-root user (for remoteUser + updateRemoteUserUID) --------------------
# IDEs that support the devcontainer spec remap this UID to match the host.
# IDEs that don't (Zed) rely on the entrypoint to detect and drop to the
# workspace owner UID via gosu.
RUN groupadd --gid 1000 dev \
    && useradd --uid 1000 --gid 1000 --no-create-home --home-dir /tmp/home --shell /bin/bash dev

# -- Permissions ---------------------------------------------------------------
# Pre-create ~/.docker: the read-only config.json bind mount would otherwise make
# Docker create the parent as root:root 0755, and buildx (which writes
# ~/.docker/buildx) then fails with "permission denied" for the runtime UID.
# Docker only creates *missing* path components, so this 1777 dir survives the mount.
RUN mkdir -p /tmp/home/.docker
RUN chmod -R 1777 /tmp/home
# /etc/passwd: entrypoint injects an entry for an arbitrary runtime UID.
# /etc/group:  entrypoint names host device/socket GIDs (Docker socket).
RUN chmod 0666 /etc/passwd /etc/group

# -- Entrypoint ----------------------------------------------------------------
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["bash"]
