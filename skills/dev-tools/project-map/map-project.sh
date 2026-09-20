#!/usr/bin/env bash
# NeoTrix Project Structure Mapper
# 生成项目结构全景视图，支持快速定位和问题发现

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/.project-map"

mkdir -p "$OUTPUT_DIR"

echo "=== NeoTrix Project Structure Mapper ==="

# Generate project structure
echo "1. 生成项目结构..."
find "$PROJECT_ROOT" -type d \
    -not -path "*/target/*" \
    -not -path "*/.git/*" \
    -not -path "*/node_modules/*" \
    -not -path "*/.project-map/*" \
    -not -path "*/.neotrix/*" \
    | sed "s|${PROJECT_ROOT}|.|g" \
    | sort > "$OUTPUT_DIR/structure.txt"

# Generate Rust module map
echo "2. 生成 Rust 模块映射..."
find "$PROJECT_ROOT/neotrix-core/src" -name "*.rs" -type f | while read -r file; do
    relative_path=$(echo "$file" | sed "s|${PROJECT_ROOT}/neotrix-core/src/||")
    module_name=$(basename "$file" .rs)
    
    echo "" >> "$OUTPUT_DIR/modules.txt"
    echo "### $relative_path" >> "$OUTPUT_DIR/modules.txt"
    echo "" >> "$OUTPUT_DIR/modules.txt"
    grep -E "^pub (fn|struct|enum|trait|mod)" "$file" 2>/dev/null | head -10 >> "$OUTPUT_DIR/modules.txt" || echo "  (无公开接口)" >> "$OUTPUT_DIR/modules.txt"
done

# Generate function index
echo "3. 生成功能索引..."
find "$PROJECT_ROOT/neotrix-core/src" -name "*.rs" -type f -exec grep -l "pub fn\|pub async fn" {} \; | while read -r file; do
    relative_path=$(echo "$file" | sed "s|${PROJECT_ROOT}/neotrix-core/src/||")
    
    echo "" >> "$OUTPUT_DIR/functions.txt"
    echo "#### $relative_path" >> "$OUTPUT_DIR/functions.txt"
    echo "" >> "$OUTPUT_DIR/functions.txt"
    grep -E "pub (async )?fn [a-zA-Z_]+" "$file" 2>/dev/null | \
        sed 's/pub (async )\{0,1\}fn /  - /' | \
        sed 's/{.*//' | \
        head -20 >> "$OUTPUT_DIR/functions.txt"
done

echo "=== 完成 ==="
echo "输出目录: $OUTPUT_DIR"
