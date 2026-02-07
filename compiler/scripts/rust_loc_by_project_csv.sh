#!/usr/bin/env bash
set -euo pipefail

# 脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# compiler 根目录（scripts 与 AADLSource 同级）
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PROJECT_ROOT="$ROOT_DIR/generate/project"
OUT_FILE="$ROOT_DIR/generate/project_rust_code_loc_by_folder.csv"

echo "folder,code_loc" > "$OUT_FILE"

declare -A SUM

# 遍历 generate/project 下所有 .rs 文件
# 统计口径：去空行 + 去注释（// 与 /* */，包含行尾注释）
while IFS= read -r -d '' file; do
  code_loc=$(
    awk '
      function trim(s){ sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); return s }

      BEGIN { in_block = 0; c = 0 }

      {
        sub(/\r$/, "", $0)
        line = $0

        # 去掉多行块注释 /* ... */（可能跨行）
        out = ""
        i = 1
        n = length(line)
        while (i <= n) {
          ch2 = substr(line, i, 2)
          if (in_block) {
            if (ch2 == "*/") { in_block = 0; i += 2; continue }
            i++
            continue
          } else {
            if (ch2 == "/*") { in_block = 1; i += 2; continue }
            if (ch2 == "//") break   # 行注释，丢弃后面
            out = out substr(line, i, 1)
            i++
          }
        }

        t = trim(out)
        if (t == "") next
        c++
      }

      END { print (c+0) }
    ' "$file"
  )

  # folder：相对 generate/project 的第一级目录（每个 case 一个文件夹）
  rel="${file#$PROJECT_ROOT/}"
  folder="${rel%%/*}"     # 取第一级目录名（case 名称）

  # 如果某些 .rs 直接落在 generate/project 根下（不太常见），归为 (root)
  if [[ "$folder" == "$rel" ]]; then
    folder="(root)"
  fi

  SUM["$folder"]=$(( ${SUM["$folder"]:-0} + code_loc ))
done < <(find "$PROJECT_ROOT" -type f -name "*.rs" -print0)

# 输出结果并排序，保证稳定
total=0
for folder in "${!SUM[@]}"; do
  echo "$folder,${SUM[$folder]}" >> "$OUT_FILE"
  total=$(( total + SUM[$folder] ))
done

{ head -n 1 "$OUT_FILE"; tail -n +2 "$OUT_FILE" | sort; } > "${OUT_FILE}.tmp"
mv "${OUT_FILE}.tmp" "$OUT_FILE"

echo "TOTAL,${total}" >> "$OUT_FILE"
echo "CSV written to $OUT_FILE"
