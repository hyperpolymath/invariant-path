#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
#
# verify-same-cube.sh — ground the AffineScript "different faces, same cube"
# invariant (invariant-path `faces` profile).
#
# Given a directory of sibling AffineScript face files that are meant to be the
# SAME program written in different faces, this compiles each face to
# typed-wasm and compares the modules. The wasm IS the cube, so byte-identical
# wasm is the rigorous bar — far stronger than diffing canonical text (text
# diffs false-positive on e.g. tail-expression vs statement lowering, which is
# observationally identical but not byte-identical).
#
# For each face it:
#   1. detects the face from the file's `face:` pragma,
#   2. compiles to wasm (`compile --face <face>`), sha256-hashes the module,
#   3. groups faces into wasm equivalence classes (canonical = reference),
#   4. for any face NOT in the canonical class, shows the `preview-*` text diff
#      as a diagnostic of WHERE the lowering diverges,
#   5. round-trip parses each face source,
#   6. prints a table + classes and (optionally) appends JSONL claim records.
#
# A break is the tool's output, not a crash: it tells you which faces fall into
# which wasm class and where. Exit 0 iff all faces share ONE wasm class; 1 if
# they split; 0 (SKIP) if no affinescript binary is reachable.
#
# Usage:
#   verify-same-cube.sh <corpus-dir> [--out FILE] [--affinescript PATH]

set -uo pipefail

CORPUS="${1:-}"; shift || true
OUT=""; AS_BIN="${AFFINESCRIPT:-}"
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

resolve_as() {
  if [ -n "$AS_BIN" ] && command -v "$AS_BIN" >/dev/null 2>&1; then echo "$AS_BIN"; return; fi
  if command -v affinescript >/dev/null 2>&1; then echo "affinescript"; return; fi
  for cand in "../affinescript/_build/default/bin/main.exe" \
              "$HOME/affinescript/_build/default/bin/main.exe"; do
    [ -x "$cand" ] && { echo "$cand"; return; }
  done
  echo ""
}
AS="$(resolve_as)"
if [ -z "$AS" ]; then
  echo "SKIP: no 'affinescript' binary found (PATH, --affinescript, AFFINESCRIPT," >&2
  echo "      or ../affinescript/_build). The same-cube invariant is grounded in" >&2
  echo "      CI where the compiler is built; nothing to verify locally." >&2
  exit 0
fi

# face pragma -> the `--face` value (canonical = default, no flag)
face_flag() {
  case "$1" in
    rattlescript|rattle|python|py)         echo "rattle" ;;
    jaffascript|jaffa|js|javascript|ts)    echo "jaffa" ;;
    pseudoscript|pseudo|pseudocode)        echo "pseudo" ;;
    lucidscript|lucid|purescript|haskell)  echo "lucid" ;;
    cafescripto|cafe|coffee|coffeescript)  echo "cafe" ;;
    canonical|affinescript|affine)         echo "CANON" ;;
    *)                                     echo "" ;;
  esac
}
preview_subcmd() {
  case "$1" in
    rattle) echo "preview-python" ;; jaffa) echo "preview-js" ;;
    pseudo) echo "preview-pseudocode" ;; lucid) echo "preview-lucid" ;;
    cafe)   echo "preview-cafe" ;; *) echo "" ;;
  esac
}
face_of() { grep -oE '(^|[[:space:]])face:[[:space:]]*[A-Za-z]+' "$1" 2>/dev/null | head -1 | sed -E 's/.*face:[[:space:]]*//'; }
normalise() { sed -E 's/[[:space:]]+/ /g; s/^ +//; s/ +$//' | grep -vE '^(//|#|--)' | grep -vE '^$'; }

program="$(basename "$CORPUS")"
[ -n "$OUT" ] && mkdir -p "$(dirname "$OUT")"
emit() { [ -n "$OUT" ] && printf '%s\n' "$1" >> "$OUT"; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

# compile a face file to wasm, echo its sha256 (empty on failure)
wasm_hash() {
  local file="$1" flag="$2"
  local out="$tmp/$(basename "$file").wasm"
  local fargs=(); [ "$flag" != "CANON" ] && [ -n "$flag" ] && fargs=(--face "$flag")
  if "$AS" compile "${fargs[@]}" "$file" -o "$out" >/dev/null 2>&1; then
    sha256sum "$out" | cut -d' ' -f1
  fi
}

# --- pass 1: locate canonical + compile every face to wasm ---------------
REF_FILE=""; REF_HASH=""
declare -A HASH FACE PARSE
order=()
for f in "$CORPUS"/*.affine; do
  [ -e "$f" ] || continue
  face="$(face_of "$f")"; flag="$(face_flag "$face")"
  [ -z "$flag" ] && continue          # unknown face pragma — skip
  base="$(basename "$f")"; order+=("$base")
  FACE[$base]="$face"
  "$AS" parse "$f" >/dev/null 2>&1 && PARSE[$base]=ok || PARSE[$base]=FAIL
  HASH[$base]="$(wasm_hash "$f" "$flag")"
  if [ "$flag" = "CANON" ]; then REF_FILE="$f"; REF_HASH="${HASH[$base]}"; fi
done
if [ -z "$REF_FILE" ]; then echo "ERROR: no 'face: canonical' reference in $CORPUS" >&2; exit 2; fi

echo "same-cube: program '$program'   (reference: $(basename "$REF_FILE"))"
echo "compiler:  $AS"
echo "bar:       byte-identical typed-wasm (the cube); preview-text diff shown for divergences"
echo "──────────────────────────────────────────────────────────"

REF_NORM="$(normalise < "$REF_FILE")"
fails=0
for base in "${order[@]}"; do
  face="${FACE[$base]}"; h="${HASH[$base]}"; p="${PARSE[$base]}"
  uri="file://$(cd "$CORPUS" && pwd)/$base"
  sub="$(preview_subcmd "$(face_flag "$face")")"
  if [ "$p" = FAIL ]; then
    printf "  %-13s %s\n" "$face" "PARSE-FAIL (source does not round-trip parse)"
    emit "{\"profile\":\"faces\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"wasm_sha256\":\"${h:-none}\",\"result\":\"Ungrounded\",\"evidence\":\"round-trip parse failed\"}"
    fails=$((fails+1)); continue
  fi
  if [ -z "$h" ]; then
    printf "  %-13s %s\n" "$face" "COMPILE-FAIL (no wasm produced)"
    emit "{\"profile\":\"faces\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"wasm_sha256\":\"none\",\"result\":\"Ungrounded\",\"evidence\":\"compile failed\"}"
    fails=$((fails+1)); continue
  fi
  if [ "$h" = "$REF_HASH" ]; then
    printf "  %-13s SAME  wasm=%s\n" "$face" "${h:0:12}"
    emit "{\"profile\":\"faces\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"wasm_sha256\":\"$h\",\"result\":\"Grounded\",\"evidence\":\"wasm byte-identical to canonical\"}"
  else
    printf "  %-13s DIFF  wasm=%s  (different cube)\n" "$face" "${h:0:12}"
    if [ -n "$sub" ]; then
      actual_norm="$("$AS" "$sub" "$CORPUS/$base" 2>/dev/null | normalise)"
      echo "    ── preview diff ($face vs canonical) ──"
      diff <(printf '%s\n' "$REF_NORM") <(printf '%s\n' "$actual_norm") | sed 's/^/    /' || true
    fi
    emit "{\"profile\":\"faces\",\"program\":\"$program\",\"face\":\"$face\",\"artifact_uri\":\"$uri\",\"wasm_sha256\":\"$h\",\"result\":\"Ungrounded\",\"evidence\":\"wasm differs from canonical cube\"}"
    fails=$((fails+1))
  fi
done

# --- wasm equivalence classes --------------------------------------------
echo "──────────────────────────────────────────────────────────"
echo "wasm equivalence classes:"
{ for base in "${order[@]}"; do [ -n "${HASH[$base]}" ] && printf '%s %s\n' "${HASH[$base]}" "${FACE[$base]}"; done; } \
  | sort | awk '{c[$1]=c[$1]" "$2} END{n=0; for(h in c){n++; printf "  class %d (%s):%s\n", n, substr(h,1,12), c[h]} exit n>1?0:0}'
nclasses="$(for base in "${order[@]}"; do [ -n "${HASH[$base]}" ] && echo "${HASH[$base]}"; done | sort -u | wc -l)"
echo "──────────────────────────────────────────────────────────"
if [ "$nclasses" -le 1 ] && [ "$fails" -eq 0 ]; then
  echo "OK: all faces compile to ONE wasm cube. Byte-level same-cube holds."
  [ -n "$OUT" ] && echo "claim records: $OUT"; exit 0
fi
echo "SPLIT: faces fall into $nclasses wasm class(es)." >&2
echo "       (Same-cube holds observationally if the classes differ only by" >&2
echo "        effect-free lowering choices, e.g. tail-expression vs statement;" >&2
echo "        byte-level same-cube requires the transformers to agree.)" >&2
exit 1
