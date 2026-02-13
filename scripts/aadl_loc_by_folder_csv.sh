#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
AADL_ROOT="$ROOT_DIR/AADLSource"

OUT_FILE="$ROOT_DIR/AADLSource/aadl_code_loc_by_folder.csv"

echo "folder,code_loc" > "$OUT_FILE"

declare -A SUM


while IFS= read -r -d '' file; do
  
  code_loc=$(
    awk '
      function trim(s){ sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); return s }
      {
        sub(/\r$/, "", $0)
        t = trim($0)

        if (t == "") next
        if (t ~ /^--/) next

        pos = index(t, "--")
        if (pos > 0) {
          before = trim(substr(t, 1, pos-1))
          if (before == "") next
        }
        c++
      }
      END { print (c+0) }
    ' "$file"
  )

  
  rel="${file#$AADL_ROOT/}"
  folder="$(dirname "$rel")"

  SUM["$folder"]=$(( ${SUM["$folder"]:-0} + code_loc ))
done < <(find "$AADL_ROOT" -type f -name "*.aadl" -print0)


total=0
for folder in "${!SUM[@]}"; do
  echo "$folder,${SUM[$folder]}" >> "$OUT_FILE"
  total=$(( total + SUM[$folder] ))
done

{ head -n 1 "$OUT_FILE"; tail -n +2 "$OUT_FILE" | sort; } > "${OUT_FILE}.tmp"
mv "${OUT_FILE}.tmp" "$OUT_FILE"

echo "TOTAL,${total}" >> "$OUT_FILE"

echo "CSV written to $OUT_FILE"
