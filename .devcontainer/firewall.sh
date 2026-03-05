#!/bin/bash
set -euo pipefail

# Network egress firewall — blocks all outbound traffic except to domains
# listed in firewall-allowlist.txt. Run as root with NET_ADMIN capability.
#
# Domain IPs are resolved once at container start. If CDN IPs rotate during
# a long-running session, restart the container to re-resolve.

ALLOWLIST_FILE="${ALLOWLIST_FILE:-/usr/local/share/firewall-allowlist.txt}"

# -- Preserve Docker internal DNS before flushing ----------------------------
docker_dns_rules=$(iptables-save | grep -E '127\.0\.0\.11' || true)

# -- Flush all rules ----------------------------------------------------------
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X

# -- Restore Docker DNS -------------------------------------------------------
if [[ -n "$docker_dns_rules" ]]; then
    echo "$docker_dns_rules" | iptables-restore --noflush
fi

# -- Build ipset from allowlist -----------------------------------------------
ipset create allowed-domains hash:net -exist
ipset flush allowed-domains

while IFS= read -r line; do
    # Strip comments and whitespace
    domain="${line%%#*}"
    domain="${domain// /}"
    [[ -z "$domain" ]] && continue

    # Resolve domain to IPs
    ips=$(dig +short A "$domain" 2>/dev/null | grep -E '^[0-9]+\.' || true)
    for ip in $ips; do
        ipset add allowed-domains "$ip/32" -exist
    done
done < "$ALLOWLIST_FILE"

# -- Default policy: DROP everything ------------------------------------------
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT DROP

# -- Allow loopback ------------------------------------------------------------
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# -- Allow established connections ---------------------------------------------
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# -- Allow DNS (required for resolution) ---------------------------------------
iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT

# -- Allow SSH (for git operations) --------------------------------------------
iptables -A OUTPUT -p tcp --dport 22 -m set --match-set allowed-domains dst -j ACCEPT

# -- Allow HTTPS to allowed domains --------------------------------------------
iptables -A OUTPUT -p tcp --dport 443 -m set --match-set allowed-domains dst -j ACCEPT

# -- Allow HTTP to allowed domains (some registries redirect) ------------------
iptables -A OUTPUT -p tcp --dport 80 -m set --match-set allowed-domains dst -j ACCEPT

# -- Reject everything else with a clear error ---------------------------------
iptables -A OUTPUT -j REJECT --reject-with icmp-admin-prohibited

# -- Self-test -----------------------------------------------------------------
if curl -sf --max-time 3 https://example.com >/dev/null 2>&1; then
    echo "FIREWALL ERROR: example.com should be blocked but is reachable" >&2
    exit 1
fi

echo "Firewall active — $(ipset list allowed-domains | grep -cE '^[0-9]') IPs allowed"
