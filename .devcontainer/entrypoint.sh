#!/bin/bash
set -euo pipefail

# -- Resolve target UID/GID ---------------------------------------------------
# Priority:
#   1. DEVCONTAINER_UID/GID env vars (firewalled mode — explicit)
#   2. Workspace directory owner (IDE started as root without explicit UID)
#   3. Current UID (normal mode — --user already set the UID)

target_uid="${DEVCONTAINER_UID:-$(id -u)}"
target_gid="${DEVCONTAINER_GID:-$(id -g)}"

# If running as root without explicit UID, infer from workspace owner.
# Handles IDEs like Zed that ignore remoteUser and start as root.
if [[ "$(id -u)" = "0" ]] && [[ -z "${DEVCONTAINER_UID:-}" ]]; then
    workspace="${DEVCONTAINER_WORKSPACE:-$(pwd)}"
    if [[ -d "$workspace" ]]; then
        target_uid="$(stat -c '%u' "$workspace")"
        target_gid="$(stat -c '%g' "$workspace")"
    fi
fi

# -- Resolve the target username, injecting a passwd entry when there is none --
# The image ships a build-time `dev` at UID 1000, so a second `dev` entry would
# make the name-based gosu drop below resolve to UID 1000 instead of the host
# UID — use a distinct name and `head -1`.
# `|| true`: getent exits 2 when the UID has NO entry (exactly the arbitrary-UID
# case handled just below). Under `set -euo pipefail` an unguarded assignment
# would abort here and the exec would never run, breaking every foreign UID.
target_user="$(getent passwd "$target_uid" | cut -d: -f1 | head -1)" || true
if [[ -z "$target_user" ]]; then
    target_user="user"
    echo "${target_user}:x:${target_uid}:${target_gid}::${HOME}:/bin/bash" >> /etc/passwd
fi

# Ensure a group with the given GID exists (creating one named $2 if not) and add
# the target user to it. Mirrors host device/socket GIDs inside the container.
# /etc/group is world-writable, so the name entry is created on any path; the
# usermod is a no-op (non-fatal) when not root.
grant_group() {  # $1 = gid, $2 = fallback group name
    getent group "$1" >/dev/null 2>&1 || echo "$2:x:$1:" >> /etc/group
    usermod -aG "$(getent group "$1" | cut -d: -f1)" "$target_user" 2>/dev/null || true
}

# -- Docker socket access (match host GID) ------------------------------------
# Ungated on purpose: on root (IDE) this adds the user to the socket group; on
# the CLI --user path the GID is already granted via --group-add, but naming the
# group here stops every shell printing "cannot find name for group ID <gid>".
if [[ -S /var/run/docker.sock ]]; then
    grant_group "$(stat -c '%g' /var/run/docker.sock)" docker
fi

# -- Firewall (firewalled mode only) -------------------------------------------
# Requires: root, NET_ADMIN capability, DEVCONTAINER_FIREWALL=1
if [[ "${DEVCONTAINER_FIREWALL:-}" = "1" ]] && [[ "$(id -u)" = "0" ]]; then
    /usr/local/bin/firewall.sh
    # Snapshot allowed IPs before privilege drop (ipset needs NET_ADMIN)
    ipset list allowed-domains | grep -E '^[0-9]' > /tmp/firewall-allowed-ips.txt 2>/dev/null || true
fi

# -- Drop privileges if running as root with a non-root target ----------------
# By USERNAME, not uid:gid — only the name form runs initgroups(), which loads
# the supplementary groups granted above (Docker socket).
if [[ "$(id -u)" = "0" ]] && [[ "${target_uid}" != "0" ]]; then
    exec gosu "${target_user}" "$@"
fi

exec "$@"
