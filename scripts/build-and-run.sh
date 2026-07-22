#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(dirname -- "$script_dir")"

unable_to_continue() {
    printf '\nUnable to continue with application build and run. Press any key to return to terminal prompt.\n'
    IFS= read -r -s -n 1 _
    printf '\n'
    exit 0
}

confirm_port_available() {
    local port="$1"
    local service="$2"

    if command -v lsof >/dev/null 2>&1; then
        local process_ids
        process_ids="$(lsof -nP -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true)"
        if [[ -n "$process_ids" ]]; then
            printf 'Cannot start %s because port %s is already in use by process ID(s): %s.\n' \
                "$service" "$port" "$(printf '%s' "$process_ids" | paste -sd ',' -)" >&2
            read -r -p 'Do you want to stop these processes and continue? (y/N) ' confirmation
            if [[ ! "$confirmation" =~ ^[Yy]$ ]]; then
                unable_to_continue
            fi
            kill $process_ids
            sleep 1
            if lsof -nP -tiTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
                printf 'Port %s is still in use after attempting to stop its processes.\n' "$port" >&2
                unable_to_continue
            fi
        fi
    elif command -v ss >/dev/null 2>&1 && ss -H -ltn "sport = :$port" 2>/dev/null | grep -q .; then
        printf 'Cannot start %s because port %s is already in use, but its process ID could not be determined.\n' "$service" "$port" >&2
        unable_to_continue
    fi
}

confirm_port_available 3000 'the Subsonic Resonance API server'

printf 'Building Subsonic Resonance application in '
for count in 3 2 1; do
    printf '%s...' "$count"
    if [[ "$count" != '1' ]]; then
        printf ' '
    fi
    sleep 1
done
printf '\n\n'

cd -- "$repository_root"
cargo build --workspace

printf '\nDo you want to:\n\n'
printf '1. Start the server and run the app\n'
printf '2. Just start the server\n\n'

while true; do
    read -r -p 'Enter 1 or 2: ' selection
    case "$selection" in
        1|2)
            break
            ;;
        *)
            printf 'Please enter 1 or 2.\n' >&2
            ;;
    esac
done

printf '\n'
if [[ "$selection" == '1' ]]; then
    confirm_port_available 3000 'the Subsonic Resonance API server'
    confirm_port_available 8088 'the Subsonic Resonance browser UI'
    printf 'Starting the API server and browser application. Press Ctrl+C to stop.\n'
    exec npm run app:start
else
    confirm_port_available 3000 'the Subsonic Resonance API server'
    printf 'Starting the API server. Press Ctrl+C to stop.\n'
    exec cargo run -p subsonic-resonance-server
fi
