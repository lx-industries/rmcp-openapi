# Dev container for rmcp-openapi
# Extends the CI image with development tools (Claude Code, glab, SSH, firewall).
#
# No WORKDIR — project mounted at host-native absolute path
# for git worktree compatibility (.git file contains absolute paths).

# -- Base: CI image (Rust toolchain + cargo-nextest, typos, sccache) -----------
# renovate: datasource=docker registryUrl=https://registry.gitlab.com depName=lx-industries/rmcp-openapi/images/rust
FROM registry.gitlab.com/lx-industries/rmcp-openapi/images/rust:1.94.0-x86_64-unknown-linux-gnu@sha256:0f94899294ed42b69e7a77b72a85b77214e70ecf415730d6f6ac9368ba7fce01

# -- Environment (set early so tool installers use the right paths) -----------
ENV HOME=/tmp/home
ENV PATH="${HOME}/.local/bin:${PATH}"

# -- NPM supply-chain hardening ------------------------------------------------
# Block postinstall scripts and require 24h package age for supply-chain safety.
ENV NPM_CONFIG_IGNORE_SCRIPTS=true
ENV NPM_CONFIG_MINIMUM_RELEASE_AGE=1440

# -- glab config directory (writable for non-root users) -----------------------
ENV GLAB_CONFIG_DIR=/tmp/glab-config

# -- System packages -----------------------------------------------------------
RUN apt-get update -qq \
    && apt-get install -y -qq --no-install-recommends \
        openssh-client \
        iptables ipset iproute2 dnsutils curl \
    && rm -rf /var/lib/apt/lists/*

# -- gosu (privilege drop for firewalled mode + IDE compatibility) -------------
# renovate: datasource=github-releases depName=tianon/gosu
ARG GOSU_VERSION="1.19"
RUN arch="$(dpkg --print-architecture)" \
    && curl -fsSL "https://github.com/tianon/gosu/releases/download/${GOSU_VERSION}/gosu-${arch}" -o /usr/local/bin/gosu \
    && chmod +x /usr/local/bin/gosu \
    && gosu --version

# -- Node.js (for integration tests — test_with_js) ---------------------------
# renovate: datasource=node-version depName=node
ARG NODE_VERSION="22.22.0"
RUN curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz" \
    | tar -xJ -C /usr/local --strip-components=1

# -- Claude Code (native binary, auto-updates) --------------------------------
# HOME is set above so the installer places the binary at $HOME/.local/bin/claude
RUN curl -fsSL https://claude.ai/install.sh | bash \
    && claude --version

# -- glab CLI (GitLab forge CLI) -----------------------------------------------
# renovate: datasource=gitlab-releases depName=gitlab-org/cli registryUrl=https://gitlab.com
ARG GLAB_VERSION="1.88.0"
RUN curl -fsSL "https://gitlab.com/gitlab-org/cli/-/releases/v${GLAB_VERSION}/downloads/glab_${GLAB_VERSION}_linux_amd64.deb" -o /tmp/glab.deb \
    && dpkg -i /tmp/glab.deb && rm /tmp/glab.deb \
    && glab --version \
    && rm -rf /tmp/glab-config \
    && mkdir -m 1777 /tmp/glab-config

# -- Git and SSH configuration -------------------------------------------------
RUN git config --system safe.directory '*'
RUN mkdir -p /etc/ssh \
    && ssh-keyscan -t ecdsa,rsa,ed25519 gitlab.com github.com >> /etc/ssh/ssh_known_hosts 2>/dev/null

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
RUN chmod -R 1777 /tmp/home
RUN chmod 0666 /etc/passwd

# -- Entrypoint ----------------------------------------------------------------
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["bash"]
