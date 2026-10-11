#!/usr/bin/env bash
# Runs an SPRT between two UCI engines with fastchess, through run_games.sh.
#
# Requires: fastchess (https://github.com/Disservin/fastchess) in the PATH.
#
# Usage: scripts/sprt.sh <candidate> <reference> <elo0> <elo1>
#   elo0, elo1   bounds in normalized Elo (fastchess default model), e.g. 0 5
#
# Environment:
#   TC           time control (default: 10+0.1)
#   ROUNDS       maximum rounds of 2 games (default: 50000)
#   CONCURRENCY  games played at once (default: physical cores - 1, at least 1)
#   FASTCHESS    the fastchess binary (default: fastchess)
#   LOG_ENGINE   true: also log every engine exchange (see run_games.sh)
#
# Exits with 0 if H1 is accepted (the candidate is better), 1 if H0 is accepted,
# 2 without a decision or on an error. A game that ends abnormally (time loss,
# illegal move, crash...) voids the result: exit 2.
#
# Example: scripts/sprt.sh target/release/aphareus ../aphareus-main 0 5

set -uo pipefail

if [[ $# -ne 4 ]]; then
    echo "usage: $0 <candidate> <reference> <elo0> <elo1>" >&2
    exit 2
fi

here=$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
output=$(mktemp)
trap 'rm -f "$output"' EXIT

"$here/run_games.sh" "$1" "$2" "${TC:-10+0.1}" "${ROUNDS:-50000}" -- \
    -sprt elo0="$3" elo1="$4" alpha=0.05 beta=0.05 | tee "$output"
status=$?

if ((status != 0)); then
    echo "SPRT: void, run_games.sh failed (status $status)" >&2
    exit 2
fi
if grep -q 'H1 was accepted' "$output"; then
    echo "SPRT: H1 accepted, the candidate is better"
    exit 0
elif grep -q 'H0 was accepted' "$output"; then
    echo "SPRT: H0 accepted, the candidate is not better"
    exit 1
fi
echo "SPRT: no decision after ${ROUNDS:-50000} rounds" >&2
exit 2
