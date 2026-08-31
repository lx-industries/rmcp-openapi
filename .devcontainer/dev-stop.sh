#!/usr/bin/env bash
# Stop every dev container launched for this project (manual fallback for a
# session the dev-shell.sh watchdog could not reap, e.g. a SIGKILLed launcher).
set -euo pipefail
project="$(basename "$(pwd)")"
ids="$(docker ps -q --filter "label=devcontainer.project=${project}")"
if [[ -z "$ids" ]]; then
    echo "No running devcontainers for '${project}'."
    exit 0
fi
echo "Stopping devcontainers for '${project}':"
docker ps --filter "label=devcontainer.project=${project}" --format '    {{.Names}}'
docker stop $ids >/dev/null
echo "Stopped $(printf '%s\n' "$ids" | grep -c .) container(s)."
