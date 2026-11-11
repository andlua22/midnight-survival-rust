#!/usr/bin/env bash
# Read-only storage report; --check gates a later build at 20 GiB free.
set -euo pipefail

if (( $# > 1 )); then
    printf 'Usage: bash tools/storage_audit.sh [--check]\n' >&2
    exit 2
fi
case "${1:-}" in
    ''|--check) ;;
    --help|-h)
        printf 'Usage: bash tools/storage_audit.sh [--check]\n'
        printf 'Reports storage without deleting files or starting a build.\n'
        printf '%s\n' '--check fails if the workspace has less than 20 GiB available.'
        exit 0
        ;;
    *)
        printf 'Usage: bash tools/storage_audit.sh [--check]\n' >&2
        exit 2
        ;;
esac

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd -- "$repo_dir"
printf 'Storage audit at %s\n' "$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
LC_ALL=C df -B1 --output=source,size,used,avail,pcent,target -- "$repo_dir" /tmp

audit_paths=()
for path in target tools/asset_unpacker/target tools/core_checks/target .local \
    assets/imported_map assets/imported_ui assets/imported_prefabs \
    assets/imported_gameplay export memory; do
    if [[ -e "$path" ]]; then
        audit_paths+=("$path")
    fi
done
du -sh -- "${audit_paths[@]}"

available_bytes="$(LC_ALL=C df -B1 --output=avail -- "$repo_dir" | awk 'NR == 2 { print $1 }')"
if [[ ! "$available_bytes" =~ ^[0-9]+$ ]]; then
    printf 'Could not read workspace free space.\n' >&2
    exit 1
fi
minimum_free_bytes=$((20 * 1024 * 1024 * 1024))
if [[ "${1:-}" == --check ]] && (( available_bytes < minimum_free_bytes )); then
    printf 'Build gate failed: %s bytes available; %s required (20 GiB).\n' \
        "$available_bytes" "$minimum_free_bytes" >&2
    exit 1
fi
if [[ "${1:-}" == --check ]]; then
    printf 'Build gate passed: at least 20 GiB available.\n'
fi
