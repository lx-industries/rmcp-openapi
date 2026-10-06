#!/usr/bin/env bash
# Writes a container-usable glab config from the host's own, on the host.
#
# Why this exists: a container cannot reach the host OS keyring, and
# `glab auth login` writes use_keyring: "true" even for a PAT login. With that
# flag on, glab reads `token` — and then `job_token` — from the keyring at API
# client init and aborts with "dbus-launch: executable file not found" before
# any token in the environment is consulted. The seed carries the token instead,
# with the flag off.
#
# PAT logins only. GitLab rotates OAuth refresh tokens single-use, so a
# container that refreshes invalidates the host's session; a host whose login is
# OAuth is skipped with a warning rather than half-authenticated.
#
# Prints the seed directory on stdout. Exits non-zero when no host could be
# seeded, so a caller can skip the mount instead of mounting an empty config.
set -euo pipefail
umask 077   # the seed carries tokens; never let a write land world-readable

source_dir="${GLAB_CONFIG_SOURCE:-$HOME/.config/glab-cli}"
seed_dir="${GLAB_SEED_DIR:-${XDG_RUNTIME_DIR:-$HOME/.cache}/glab-devcontainer}"

mkdir -p "$seed_dir"
chmod 700 "$seed_dir"

# The seed directory is mounted read-only over GLAB_CONFIG_DIR, and an EMPTY one
# is worse than no mount at all: glab writes its default config before any read,
# fails on the read-only mount, and every command dies with "failed to read
# configuration: ... permission denied". So always leave a config behind, even
# when there is nothing to seed.
write_fallback_config() {
    printf 'git_protocol: ssh\nhosts:\n' > "$seed_dir/config.yml"
    chmod 600 "$seed_dir/config.yml"
}

# Nothing to seed is not a failure worth printing: a host without glab, or
# without a glab config, would otherwise emit this on every container launch.
# The caller reads the exit status and skips the mount.
[[ -f "$source_dir/config.yml" ]] || { write_fallback_config; exit 1; }
command -v glab >/dev/null 2>&1 || { write_fallback_config; exit 1; }

token_map="$(mktemp)"
trap 'rm -f "$token_map"' EXIT INT TERM

hosts="$(awk '
    /^hosts:$/ { in_hosts = 1; next }
    in_hosts && /^[^ ]/ { in_hosts = 0 }
    in_hosts && /^    [^ ].*:$/ { line = $0; sub(/^    /, "", line); sub(/:$/, "", line); print line }
' "$source_dir/config.yml")"

# A host block that still carries OAuth material is an OAuth login, whatever the
# token looks like. The gloas- prefix catches the case where that material lives
# in the keyring and never appears in the file.
host_is_oauth() {
    awk -v host="$1" '
        /^    [^ ].*:$/ { current = $0; sub(/^    /, "", current); sub(/:$/, "", current) }
        current == host && /^        oauth2_(refresh_token|expiry_date): ./ { found = 1 }
        END { exit(found ? 0 : 1) }
    ' "$source_dir/config.yml"
}

host_token_from_file() {
    awk -v host="$1" '
        /^    [^ ].*:$/ { current = $0; sub(/^    /, "", current); sub(/:$/, "", current) }
        current == host && /^        token: ./ { line = $0; sub(/^        token: /, "", line); print line; exit }
    ' "$source_dir/config.yml"
}

seeded=0
for host in $hosts; do
    if host_is_oauth "$host"; then
        echo "glab-seed: skipping $host — OAuth login; containers need a PAT" >&2
        continue
    fi
    # `glab config get token --host X` returns $GITLAB_TOKEN / $OAUTH_TOKEN for
    # ANY X when those are exported — the environment outranks the config — so a
    # launcher run from a shell that exports one would stamp that single token
    # into every host block and send it to hosts it does not belong to.
    token="$(env -u GITLAB_TOKEN -u GITLAB_ACCESS_TOKEN -u OAUTH_TOKEN \
        glab config get token --host "$host" 2>/dev/null || true)"
    # A keyring read can fail where the file still holds a usable token; prefer
    # keeping that over blanking it.
    [[ -z "$token" ]] && token="$(host_token_from_file "$host")"
    # `-h` is --help for `glab config get`; a multi-word capture is help text.
    case "$token" in *[[:space:]]*) token="" ;; esac
    case "$token" in
        "")        echo "glab-seed: skipping $host — no token on the host" >&2; continue ;;
        gloas-*)   echo "glab-seed: skipping $host — OAuth login; containers need a PAT" >&2; continue ;;
    esac
    printf '%s\t%s\n' "$host" "$token" >> "$token_map"
    seeded=$((seeded + 1))
done

# Copy the host config with three edits per host block: the token inlined for a
# seeded host and blanked for a skipped one, the keyring turned off everywhere
# (a skipped host then returns 401 instead of aborting the whole client), and
# every oauth2_* line dropped so no refresh material can reach the container.
awk -v token_map="$token_map" '
    BEGIN {
        while ((getline line < token_map) > 0) {
            split(line, field, "\t")
            token[field[1]] = field[2]
        }
    }
    { sub(/\r$/, "") }
    /^hosts:$/ { in_hosts = 1; print; next }
    in_hosts && /^[^ ]/ { in_hosts = 0; host = "" }
    in_hosts && /^    [^ ].*:$/ {
        host = $0; sub(/^    /, "", host); sub(/:$/, "", host)
        print
        if (host in token) print "        token: " token[host]
        next
    }
    in_hosts && host != "" && /^        oauth2_/ { next }
    in_hosts && host != "" && /^        token:/ { next }
    in_hosts && host != "" && /^        use_keyring:/ { print "        use_keyring: \"false\""; next }
    { print }
' "$source_dir/config.yml" > "$seed_dir/config.yml"
chmod 600 "$seed_dir/config.yml"

if [[ -f "$source_dir/aliases.yml" ]]; then
    cp "$source_dir/aliases.yml" "$seed_dir/aliases.yml"
    chmod 600 "$seed_dir/aliases.yml"
fi

(( seeded > 0 )) || {
    # The transform may have produced a verbatim copy (an unparsable config), and
    # devcontainer.json mounts this directory whether or not seeding worked — a
    # copy carrying use_keyring: "true" would reintroduce the dbus-launch abort.
    write_fallback_config
    echo "glab-seed: no host could be seeded; the container will have no glab auth" >&2
    exit 1
}

printf '%s\n' "$seed_dir"
