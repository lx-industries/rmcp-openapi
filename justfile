# Launch an interactive devcontainer shell (e.g. `just dev-shell`, `just dev-shell claude`).
# Add --firewall for network-firewalled autonomous mode.
[positional-arguments]
dev-shell *args:
    @.devcontainer/dev-shell.sh "$@"

# Stop any devcontainer left running for this project.
dev-stop:
    @.devcontainer/dev-stop.sh
