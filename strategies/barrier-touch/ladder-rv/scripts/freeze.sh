#!/usr/bin/env bash
# Daily freeze for barrier-touch/ladder-rv, with the tar line VERIFIED instead of trusted.
#
# Why this script exists. On 2026-07-28 the daily freeze tarred `candles vol` and nothing
# else. `data/out/` is gitignored, so `predictions_2026-07-28.csv` -- the per-leg record the
# RV/IV pre-registration is scored from -- was frozen NOWHERE and survived in one container
# only. `r2data verify` returned OK every day, because it HEADs the blob and cannot see
# inside a tarball. The failure was not in code: it was a hand-written `tar` line that
# omitted a directory, re-typed every morning.
# On 2026-07-30 a second instance was found: `predictions_2026-07-26.csv` is in no archive
# at all and is permanently lost, because day 4 cut no `live-*` freeze.
#
# So: the manifest of what must be frozen lives HERE, in git, and the script re-reads the
# tarball it just built and fails if anything it promised is missing.
#
#   usage: scripts/freeze.sh <YYYY-MM-DD>
#
# Cuts BOTH archives every day -- candles-<date> (resolution record) and live-<date>
# (per-leg record). Neither is optional; one without the other is how day 6 broke.
set -euo pipefail

DATE="${1:?usage: freeze.sh YYYY-MM-DD}"
cd "$(dirname "$0")/.."
DATA=data
R2=../../../tools/r2data/target/release/r2data
[[ -x "$R2" ]] || { echo "build r2data first: (cd tools/r2data && cargo build --release)"; exit 1; }

# --- what each archive MUST contain. Add a line here, not to a tar command. ---
CANDLES_PATHS=(candles vol)
CANDLES_MUST=(candles/WTIU6 candles/WTIV6 candles/XAUUSD candles/XAGUSD candles/SPY
              candles/NVDA candles/USOILSPOT candles/BTCUSDT candles/ETHUSDT
              vol/ovx.csv vol/gvz.csv vol/vxslv.csv vol/vix.csv)
# `tape/` and `clob*/` are gitignored too. The tape files are the ONLY evidence behind a
# tape-gate suppression (2026-07-29 suppressed gold-weekly dip-to-3950 on them), and
# `clob60/` is what cmd_analyze reads for every checkpoint Brier. Neither belongs only in
# a container.
LIVE_PATHS=(events_live out legs.csv tape clob60 clob10)
LIVE_MUST=(out/predictions_"${DATE}".csv legs.csv events_live)

freeze() {
  local name="$1"; shift
  local -n paths=$1; shift
  local -n must=$1; shift
  local tgz="$DATA/$name.tar.gz"

  local present=()
  for p in "${paths[@]}"; do [[ -e "$DATA/$p" ]] && present+=("$p"); done
  [[ ${#present[@]} -gt 0 ]] || { echo "FAIL $name: none of ${paths[*]} exist"; return 1; }

  tar czf "$tgz" -C "$DATA" "${present[@]}"

  # VERIFY THE CONTENTS, not the existence. This is the whole point of the script.
  local listing; listing="$(tar tzf "$tgz")"
  local missing=0
  for m in "${must[@]}"; do
    if ! grep -qE "(^|/)${m}(/|$)" <<<"$listing"; then
      echo "FAIL $name: required entry MISSING from the tarball: $m"; missing=1
    fi
  done
  # a 52-byte Pyth `no_data` stub or a 2-byte empty Binance array is not data
  local nstub
  nstub="$(tar tzvf "$tgz" | awk '$3<60 && $NF ~ /\.json$/' | wc -l | tr -d ' ')"
  [[ "$nstub" != "0" ]] && echo "  note $name: $nstub json entries under 60B (weekend / pre-session stubs are expected; a WEEKDAY session stub is not)"
  [[ $missing -eq 0 ]] || return 1

  echo "  $name: $(tar tzf "$tgz" | wc -l) entries, $(du -h "$tgz" | cut -f1)"
  "$R2" push "$tgz" >/dev/null
  "$R2" verify "$tgz.r2.json"
  rm -f "$tgz"     # the manifest is the tracked record; the blob lives in R2
}

echo "freeze candles-$DATE"
freeze "candles-$DATE" CANDLES_PATHS CANDLES_MUST
echo "freeze live-$DATE"
freeze "live-$DATE" LIVE_PATHS LIVE_MUST
echo
echo "Both archives cut. Commit data/*.r2.json AFTER this succeeds (AGENTS.md: upload before"
echo "committing the manifest). Remember: verify checks the blob, not the contents -- and a"
echo "verify FAIL can be a transient R2 500, so retry a FAIL with 'r2data pull' before"
echo "concluding an archive is lost."
