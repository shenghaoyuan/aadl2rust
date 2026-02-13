cat > scripts/aadl_loc_code_csv.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

ROOT="AADLSource"
OUT_DIR="tables"
OUT_FILE="$OUT_DIR/aadl_code_loc_by_dir.csv"

mkdir -p "$OUT_DIR"
echo "model,code_loc" > "$OUT_FILE"

find "$ROOT" -type f -name "*.aadl" -print0 \
| awk -v RS='\0' -v root="$ROOT" '
  function trim(s) { sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); return s }

  {
    file = $0

    dir = file
    sub("^" root "/", "", dir)
    sub(/\/[^\/]+$/, "", dir)

    code = 0
    while ((getline line < file) > 0) {
      sub(/\r$/, "", line)
      t = trim(line)

      if (t == "") continue          # 空行
      if (t ~ /^--/) continue        # 整行注释（AADL: -- ...）

      pos = index(t, "--")           # 行尾注释
      if (pos > 0) {
        before = trim(substr(t, 1, pos-1))
        if (before == "") continue
      }

      code++
    }
    close(file)

    sum[dir] += code
  }
  END {
    total = 0
    for (d in sum) {
      printf "%s,%d\n", d, sum[d]
      total += sum[d]
    }
    printf "TOTAL,%d\n", total
  }
' >> "$OUT_FILE"

echo "CSV written to $OUT_FILE"
EOF

chmod +x scripts/aadl_loc_code_csv.sh
