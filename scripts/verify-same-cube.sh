#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
#
# verify-same-cube.sh — ground the AffineScript "different faces, same cube"
# invariant (invariant-path `faces` profile).
#
# Given a directory of sibling AffineScript face files that are meant to be
# the SAME program written in different faces, this:
#   1. detects each file's face from its `face:` pragma,
#   2. runs the matching `preview-*` transformer to get canonical text,
#   3. normalises (drops comment-only lines, blank lines, collapses
#      whitespace) and compares each face's lowering against the canonical
#      reference (canonical.affine / the `face: canonical` file),
#   4. round-trip parses each face file,
#   5. prints a table and (optionally) appends JSONL claim records.
#
# A break is not a crash — it is the tool's output: it tells you WHICH face
# diverges from the cube. Exit 1 on any divergence/parse failure, 0 when the
# cube holds (or when no affinescript binary is available to ground it).
#
# Usage:
#   verify-same-cube.sh <corpus-dir> [--out FILE] [--affinescript PATH]
#
# Example:
#   verify-same-cube.sh examples/same-cube/greet --out /tmp/same-cube.jsonl

set -uo pipefail

CORPUS="${1:-}"
shift || true
OUT=""
AS_BIN="${AFFINESCRIPT:-}"
while [ $# -gt 0 ]; do
  case "$1" in
    --out)          OUT="$2"; shift 2 ;;
    --affinescript) AS_BIN="$2"; shift 2 ;;
    -h|--help)      sed -n '3,30p' "$0"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [ -z "$CORPUS" ] || [ ! -d "$CORPUS" ]; then
  echo "usage: verify-same-cube.sh <corpus-dir> [--out FILE] [--affinescript PATH]" >&2
  exit 2
fi

# --- resolve the affinescript compiler ------------------------------------
resolve_as() {
  if [ -n "$AS_BIN" ] && command -v "$AS_BIN" >/dev/null 2>&1; then echo "$AS_BIN"; return; fi
  if command -v affinescript >/dev/null 2>&1; then echo "affinescript"; return; fi
  for cand in \
    "../affinescript/_build/default/bin/main.exe" \
    "$HOME/affinescript/_build/default/bin/main.exe"; do
    [ -x "$cand" ] && { echo "$cand"; return; }
  done
  echo ""  # not found
}
AS="$(resolve_as)"

if [ -z "$AS" ]; then
  echo "SKIP: no 'affinescript' binary found (PATH, --affinescript, AFFINESCRIPT," >&2
  echo "      or ../affinescript/_build). The same-cube invariant is grounded in" >&2
  echo "      CI where the compiler is built; nothing to verify locally." >&2
  exit 0
fi

# --- face pragma -> preview subcommand ------------------------------------
preview_subcmd() {
  case "$1" in
    rattlescript|rattle|python|py)            echo "preview-python" ;;
    jaffascript|jaffa|js|javascript|ts)       echo "preview-js" ;;
    pseudoscript|pseudo|pseudocode)           echo "preview-pseudocode" ;;
    lucidscript|lucid|purescript|haskell)     echo "preview-lucid" ;;
    cafescripto|cafe|coffee|coffeescript)     echo "preview-cafe" ;;
    canonical|affinescript|affine)            echo "CANONICAL" ;;
    *)                                        echo "" ;;
  esac
}

face_of() {  # read the `face:` pragma from a file
  grep -oE '(^|[[:space:]])face:[[:space:]]*[A-Za-z]+' "$1" 2>/dev/null \
    | head -1 | sed -E 's/.*face:[[:space:]]*//'
}

normalise() {  # canonical text -> comparable form (modulo comments + whitespace)
  sed -E 's/[[:space:]]+/ /g; s/^ +//; s/ +$//' \
    | grep -vE '^(//|#|--)' \
    | grep -vE '^$'
}

# --- locate the canonical reference ---------------------------------------
REF_FILE=""
for f in "$CORPUS"/*.affine; do
  [ -e "$f" ] || continue
  [ "$(preview_subcmd "$(face_of "$f")")" = "CANONICAL" ] && REF_FILE="$f"
done
if [ -z "$REF_FILE" ]; then
  echo "ERROR: no canonical reference (a file with 'face: canonical') in $CORPUS" >&2
  exit 2
fi
REF_NORM="$(normalise < "$REF_FILE")"

program="$(basename "$CORPUS")"
[ -n "$OUT" ] && mkdir -p "$(dirname "$OUT")"
emit() { [ -n "$OUT" ] && printf '%s\n' "$1" >> "$OUT"; }

echo "same-cube: program '$program'  (reference: $(basename "$REF_FILE"))"
echo "compiler: $AS"
echo "──────────────────────────────────────────────"

fails=0; checks=0
# canonical must parse
if ! "$AS" parse "$REF_FILE" >/dev/null 2>&1; then
  printf "  %-12s %s\n" "canonical" "FAIL (reference does not parse)"
  fails=$((fails+1))
fi

for f in "$CORPUS"/*.affine; do
  [ -e "$f" ] || continue
  face="$(face_of "$f")"
  sub="$(preview_subcmd "$face")"
  [ "$sub" = "CANONICAL" ] && continue
  if [ -z "$sub" ]; then
    printf "  %-12s %s\n" "$face" "SKIP (unknown face pragma in $(basename "$f"))"
    continue
  fi
  checks=$((checks+1))
  uri="file://$(cd "$(dirname "$f")" && pwd)/$(basename "$f")"

  # round-trip parse of the face source (auto-detects face via pragma)
  parse_ok=1
  "$AS" parse "$f" >/dev/null 2>&1 || parse_ok=0

  # lower via preview-* and compare to the canonical cube
  actual_norm="$("$AS" "$sub" "$f" 2>/dev/null | normalise)"
  if [ "$parse_ok" -eq 0 ]; then
    printf "  %-12s %s\n" "$face" "FAIL (face source does not round-trip parse)"
    emit "{\"profile\":\"faces\",\"claim\":\"different faces, same cube\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"transition\":\"$sub\",\"target\":\"$(basename "$REF_FILE")\",\"result\":\"Ungrounded\",\"evidence\":\"round-trip parse failed\"}"
    fails=$((fails+1))
  elif [ "$actual_norm" = "$REF_NORM" ]; then
    printf "  %-12s %s\n" "$face" "OK   (lowers to the same cube)"
    emit "{\"profile\":\"faces\",\"claim\":\"different faces, same cube\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"transition\":\"$sub\",\"target\":\"$(basename "$REF_FILE")\",\"result\":\"Grounded\",\"evidence\":\"normalised canonical equals reference\"}"
  else
    printf "  %-12s %s\n" "$face" "DIFF (lowers to a DIFFERENT cube)"
    echo "    ── canonical lowering diff ($face vs canonical) ──"
    diff <(printf '%s\n' "$REF_NORM") <(printf '%s\n' "$actual_norm") | sed 's/^/    /' || true
    emit "{\"profile\":\"faces\",\"claim\":\"different faces, same cube\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"transition\":\"$sub\",\"target\":\"$(basename "$REF_FILE")\",\"result\":\"Ungrounded\",\"evidence\":\"normalised canonical differs from reference\"}"
    fails=$((fails+1))
  fi
done

echo "──────────────────────────────────────────────"
if [ "$fails" -gt 0 ]; then
  echo "FAIL: $fails of $checks face(s) diverge from the cube." >&2
  exit 1
fi
echo "OK: all $checks face(s) lower to the same cube. Invariant holds."
[ -n "$OUT" ] && echo "claim records: $OUT"
exit 0
