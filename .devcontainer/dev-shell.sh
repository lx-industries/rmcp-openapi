#!/usr/bin/env bash
# Launch the rmcp-openapi dev container.
#
#   .devcontainer/dev-shell.sh                 # interactive shell
#   .devcontainer/dev-shell.sh claude          # run a command inside
#   .devcontainer/dev-shell.sh --firewall claude   # firewalled autonomous mode
#
# Kept as a plain script (not inline in the justfile) so it is testable on its
# own, usable without `just`, and free of just's {{ }} brace escaping.
set -euo pipefail

image=rmcp-openapi-devcontainer
docker build -t "$image" .devcontainer/

project="$(basename "$(pwd)")"
tty_flag=$( [[ -t 0 ]] && echo "-it" || echo "-i" )
run_args=(
    --rm $tty_flag --init
    --label "devcontainer.project=$project"
    -v "$(pwd):$(pwd)" -w "$(pwd)"
    -e COLORTERM="${COLORTERM:-}"
    -e GIT_SSH_COMMAND="ssh -o UserKnownHostsFile=/etc/ssh/ssh_known_hosts"
    -e DEVCONTAINER_WORKSPACE="$(pwd)"
)

# SSH-agent forwarding is best-effort: an SSH login without agent forwarding has
# no SSH_AUTH_SOCK, and under `set -u` an unconditional mount would abort here.
if [[ -n "${SSH_AUTH_SOCK:-}" ]]; then
    run_args+=(
        -v "$SSH_AUTH_SOCK:/tmp/ssh-agent.sock"
        -e SSH_AUTH_SOCK=/tmp/ssh-agent.sock
    )
fi

# Leading flags (stop at the first non-flag token so the pass-through command
# keeps its own flags).
firewall=0
while [[ "${1:-}" == --* ]]; do
    case "$1" in
        --firewall) firewall=1 ;;
        *) break ;;
    esac
    shift
done

# Firewalled mode: iptables egress filter, start as root, drop to the host UID
# via gosu in the entrypoint. Normal mode: run directly as the host UID.
if [[ $firewall -eq 1 ]]; then
    run_args+=(
        --cap-add=NET_ADMIN --cap-add=NET_RAW
        -e DEVCONTAINER_FIREWALL=1
        -e DEVCONTAINER_UID="$(id -u)"
        -e DEVCONTAINER_GID="$(id -g)"
    )
else
    run_args+=(--user "$(id -u):$(id -g)")
fi

# -- Conditional host config mounts -------------------------------------------
[[ -f "$HOME/.gitconfig" ]] && run_args+=(-v "$HOME/.gitconfig:/tmp/home/.gitconfig:ro")
[[ -d "$HOME/.config/glab-cli" ]] && run_args+=(-v "$HOME/.config/glab-cli:/tmp/glab-config")
[[ -d "$HOME/.config/gh" ]] && run_args+=(-v "$HOME/.config/gh:/tmp/gh-config")
[[ -d "$HOME/.claude" ]] && run_args+=(
    -v "$HOME/.claude:/tmp/home/.claude"
    -v "$HOME/.claude:$HOME/.claude"
)
[[ -f "$HOME/.claude.json" ]] && run_args+=(-v "$HOME/.claude.json:/tmp/home/.claude.json")

# gh keeps its token in the OS keyring, which cannot cross the container
# boundary; forward it with the pass-through form (-e GH_TOKEN, no inline value)
# so the secret stays out of the docker argv. glab is NOT forwarded — its OAuth
# access token does not round-trip as a PRIVATE-TOKEN; log in with a PAT so the
# token lives in the mounted ~/.config/glab-cli instead.
if command -v gh >/dev/null 2>&1; then
    GH_TOKEN="$(gh auth token 2>/dev/null || true)"; export GH_TOKEN
    [[ -n "$GH_TOKEN" ]] && run_args+=(-e GH_TOKEN)
fi

# Docker socket — the host daemon does the work; no daemon runs in-container.
if [[ -S /var/run/docker.sock ]]; then
    run_args+=(-v /var/run/docker.sock:/var/run/docker.sock)
    run_args+=(--group-add "$(stat -c '%g' /var/run/docker.sock)")
fi
# Host registry credentials (read-only). Mount ONLY config.json — a read-only
# mount of the whole ~/.docker dir breaks buildx, which writes to ~/.docker/buildx.
[[ -f "$HOME/.docker/config.json" ]] && run_args+=(-v "$HOME/.docker/config.json:/tmp/home/.docker/config.json:ro")

# Superpowers brainstorming visual companion — publish a free host port.
_find_free_port() {
    local port="${1:-19452}"
    local max=$((port + 100))
    while ss -tlnH "sport = :${port}" 2>/dev/null | grep -q .; do
        port=$((port + 1))
        if (( port > max )); then
            echo "ERROR: no free port in range ${1:-19452}-${max}" >&2
            return 1
        fi
    done
    echo "$port"
}
BRAINSTORM_PORT="${BRAINSTORM_PORT:-$(_find_free_port 19452)}"
run_args+=(
    -p "${BRAINSTORM_PORT}:${BRAINSTORM_PORT}"
    -e "BRAINSTORM_PORT=${BRAINSTORM_PORT}"
    -e "BRAINSTORM_HOST=0.0.0.0"
    -e "BRAINSTORM_URL_HOST=localhost"
)

# -- Container name (unique across worktrees and repeat launches) -------------
base_name="${project}-devcontainer"
container_name="$base_name"; n=2
while docker ps -a --format '{{.Names}}' 2>/dev/null | grep -qx "$container_name"; do
    container_name="${base_name}-$n"; n=$((n + 1))
done
run_args+=(--name "$container_name")

# -- Reap on terminal death (interactive only) --------------------------------
# A foreground `docker run -it` does not return when the terminal closes, and an
# EXIT trap loses the race against the shell's own SIGHUP. So: a watchdog that
# ignores SIGHUP, is disowned, and polls the controlling terminal by path. Once
# writes fail the terminal is gone, so it stops the container and --rm collects
# it. SIGKILL of the launcher is untrappable — use `just dev-stop` for that.
if [[ -t 0 ]]; then
    tty_path="$(tty)"
    ( trap '' HUP
      for _ in $(seq 1 40); do
          { : >"$tty_path"; } 2>/dev/null || { docker stop -t 5 "$container_name" >/dev/null 2>&1; exit 0; }
          docker inspect "$container_name" >/dev/null 2>&1 && break
          sleep 0.5
      done
      while { : >"$tty_path"; } 2>/dev/null; do
          docker inspect "$container_name" >/dev/null 2>&1 || exit 0
          sleep 0.5
      done
      docker stop -t 5 "$container_name" >/dev/null 2>&1 || true ) &
    disown 2>/dev/null || true
fi

# Report leftovers on exit, not on entry: at launch a sibling container is the
# normal multi-worktree case, but anything still up after the run returns leaked.
report_leftovers() {
    local left line out=""
    left="$(docker ps --filter "label=devcontainer.project=${project}" \
        --format '{{.Names}} (up {{.RunningFor}})' 2>/dev/null || true)"
    while IFS= read -r line; do
        [[ -n "$line" ]] || continue
        [[ "${line%% *}" == "$container_name" ]] && continue   # our own, --rm is collecting it
        out+="     ${line}"$'\n'
    done <<<"$left"
    [[ -n "$out" ]] || return 0
    {
        echo "⚠  Devcontainer(s) for '${project}' still running:"
        printf '%s' "$out"
        echo "     Attach:  docker attach <name>"
        echo "     Stop:    just dev-stop"
    } >&2
}

status=0
if [[ $# -eq 0 ]]; then
    docker run "${run_args[@]}" "$image" bash || status=$?
else
    docker run "${run_args[@]}" "$image" "$@" || status=$?
fi
report_leftovers
exit $status
