#!/usr/bin/env bash
# Plays a match between two UCI engines with fastchess.
#
# Requires: fastchess (https://github.com/Disservin/fastchess) in the PATH.
#
# Usage: scripts/run_games.sh <engine1> <engine2> [tc] [rounds] [-- fastchess args...]
#   tc       time control, fastchess format (default: 10+0.1)
#   rounds   rounds of 2 games on the same opening, colors swapped (default: 200)
#   -- ...   extra arguments passed to fastchess as they are (e.g. -strict)
#
# Environment:
#   CONCURRENCY  games played at once (default: physical cores - 1, at least 1)
#   FASTCHESS    the fastchess binary (default: fastchess)
#   LOG_ENGINE   true: also log every engine exchange (large: ~3400 lines per game)
#
# Writes to target/games/<timestamp>: the games (.pgn), the fastchess warnings (.log)
# and the fastchess state to resume an interrupted match (.json).
# Exits with 1 if fastchess fails, a game ends abnormally (time loss, illegal move,
# crash...) or, without -sprt, fewer games than asked were played. Exits with 2 on a
# usage error.
#
# Example: scripts/run_games.sh target/release/aphareus target/release/aphareus 1+0.01 100

set -euo pipefail

usage() {
    echo "usage: $0 <engine1> <engine2> [tc] [rounds] [-- fastchess args...]" >&2
    exit 2
}

physical_cores() {
    local cores=""
    if command -v lscpu >/dev/null; then
        cores=$(LC_ALL=C lscpu -p=Core,Socket | grep -v '^#' | sort -u | wc -l)
    elif command -v sysctl >/dev/null; then
        cores=$(sysctl -n hw.physicalcpu 2>/dev/null || true)
    fi
    if [[ -z "$cores" || "$cores" -lt 1 ]]; then
        cores=$(getconf _NPROCESSORS_ONLN)
        echo "warning: physical cores unknown, using $cores logical CPUs; set CONCURRENCY" >&2
    fi
    echo "$cores"
}

[[ $# -ge 2 ]] || usage
engine1=$1
engine2=$2
shift 2
tc=10+0.1
rounds=200
if [[ $# -gt 0 && $1 != -- ]]; then tc=$1; shift; fi
if [[ $# -gt 0 && $1 != -- ]]; then rounds=$1; shift; fi
if [[ $# -gt 0 ]]; then
    [[ $1 == -- ]] || usage
    shift
fi

fastchess=${FASTCHESS:-fastchess}
command -v "$fastchess" >/dev/null || {
    echo "fastchess not found: install it or set FASTCHESS" >&2
    exit 2
}
for engine in "$engine1" "$engine2"; do
    command -v "$engine" >/dev/null || {
        echo "engine not found or not executable: $engine" >&2
        exit 2
    }
done

# Keep a core free for the machine.
concurrency=${CONCURRENCY:-$(($(physical_cores) - 1))}
((concurrency >= 1)) || concurrency=1

# fastchess needs two distinct names.
name1=$(basename "$engine1")
name2=$(basename "$engine2")
[[ $name1 != "$name2" ]] || name2=$name2-2

log_options=(level=warn)
if [[ ${LOG_ENGINE:-false} == true ]]; then
    log_options+=(engine=true)
fi

sprt=false
for arg in "$@"; do
    if [[ $arg == -sprt ]]; then sprt=true; fi
done

root=$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
out=$root/target/games
mkdir -p "$out"
stamp=$(date +%Y%m%d-%H%M%S)-$$
pgn=$out/$stamp.pgn
log=$out/$stamp.log

echo "$name1 vs $name2, tc $tc, $rounds rounds, concurrency $concurrency"
status=0
"$fastchess" \
    -engine cmd="$engine1" name="$name1" \
    -engine cmd="$engine2" name="$name2" \
    -each tc="$tc" proto=uci option.Threads=1 option.Hash=16 \
    -openings file="$root/scripts/openings/book.epd" format=epd order=random \
    -rounds "$rounds" -repeat \
    -concurrency "$concurrency" \
    -pgnout file="$pgn" \
    -log file="$log" "${log_options[@]}" \
    -config outname="$out/$stamp.json" \
    "$@" || status=$?

echo "PGN: $pgn"
warnings=$(grep -Ec '^\[(WARN|ERR|FATAL)' "$log" 2>/dev/null || true)
if ((${warnings:-0} > 0)); then
    echo "fastchess warnings: $warnings in $log"
fi
[[ -f $pgn ]] || {
    echo "no game was played (fastchess status $status)" >&2
    exit 1
}

echo "Terminations:"
grep '^\[Termination ' "$pgn" | sort | uniq -c || true
abnormal=$(grep '^\[Termination ' "$pgn" | grep -vc -e '"normal"' -e '"adjudication"' || true)
if ((abnormal > 0)); then
    echo "$abnormal games ended abnormally" >&2
    exit 1
fi
if ((status != 0)); then
    echo "fastchess failed with status $status" >&2
    exit 1
fi
played=$(grep -c '^\[Result ' "$pgn" || true)
if [[ $sprt == false ]] && ((played != 2 * rounds)); then
    echo "$played games played instead of $((2 * rounds))" >&2
    exit 1
fi
