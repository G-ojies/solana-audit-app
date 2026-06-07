#!/usr/bin/env bash
# Ghost Protocol scheduled watcher.
# Runs the agent's `watch` command and raises a desktop notification only when
# genuinely-open, skill-matched Superteam listings appear. Always appends a
# timestamped verdict to ghost_watch.log. Intended to be run from cron.

set -uo pipefail

PROJECT_DIR="/home/greyat_labs/Development/ghost-protocol"
LOG="$PROJECT_DIR/ghost_watch.log"
cd "$PROJECT_DIR" || exit 1

# "heartbeat" mode sends a Telegram status even when there are 0 open listings,
# so the user has positive confirmation the watcher is alive.
MODE="${1:-watch}"

# Make desktop notifications work from a cron environment.
export DISPLAY="${DISPLAY:-:0}"
export DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-unix:path=/run/user/$(id -u)/bus}"

TS="$(date '+%Y-%m-%d %H:%M:%S')"
OUT="$(python3 ghost_protocol.py watch 2>>"$LOG")"

# The watch command prints a single line beginning with GHOST_WATCH:
VERDICT="$(printf '%s\n' "$OUT" | grep -m1 'GHOST_WATCH:')"
echo "[$TS] ${VERDICT:-no verdict line (see errors above)}" >>"$LOG"

# Notify only when there are OPEN listings (verdict is not the "0 open" message).
if [ -n "$VERDICT" ] && ! printf '%s' "$VERDICT" | grep -q '0 open listings'; then
    # Append the detailed listing bullets to the log for review.
    printf '%s\n' "$OUT" | grep -E '^\s+•' >>"$LOG"
    COUNT="$(printf '%s' "$VERDICT" | grep -oE '[0-9]+' | head -1)"
    notify-send -u critical "Ghost Protocol 🔔" \
        "${COUNT:-Some} open Superteam listing(s) found. See $LOG and run a submission." \
        2>>"$LOG" || true

    # Email alert (no-op if SMTP_* not configured in .env).
    {
        printf '%s open Superteam listing(s) matched Ghost Protocol.\n\n' "${COUNT:-Some}"
        printf '%s\n' "$OUT" | grep -E '^\s+•'
        printf '\nSubmit the onchain-rbac deliverable with:\n'
        printf '  python3 ghost_protocol.py submit --slug <slug> --link https://github.com/G-ojies/solana-audit-app/tree/master/Development/onchain-rbac --info "..."\n'
    } | python3 ghost_notify.py --subject "Ghost Protocol: ${COUNT:-some} open listing(s) 🔔" 2>>"$LOG" || true

elif [ "$MODE" = "heartbeat" ]; then
    # Positive liveness ping regardless of result.
    printf '%s' "Ghost Protocol watcher is alive ✅
${VERDICT:-no verdict}" \
        | python3 ghost_notify.py --subject "Ghost Protocol — weekly heartbeat 💓" 2>>"$LOG" || true
fi
