#!/bin/bash

MONITOR_DIR="/Users/neo/Downloads/code/neotrix"
LOG_FILE="$MONITOR_DIR/.monitor_log.txt"
PREV_ERROR_HASH=""

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Background monitor started (PID: $$)" >> "$LOG_FILE"

while true; do
  TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

  # 1. Cargo check
  CHECK_OUTPUT=$(cd "$MONITOR_DIR" && cargo check --lib --offline 2>&1 | tail -5)
  CHECK_EXIT=$?

  CURRENT_ERROR_HASH=$(echo "$CHECK_OUTPUT" | md5)

  if echo "$CHECK_OUTPUT" | grep -q "error"; then
    echo "[$TIMESTAMP] [ERROR] cargo check failed:" >> "$LOG_FILE"
    echo "$CHECK_OUTPUT" | while IFS= read -r line; do
      echo "  $line" >> "$LOG_FILE"
    done
  elif [ "$PREV_ERROR_HASH" != "$CURRENT_ERROR_HASH" ] && [ -n "$CHECK_OUTPUT" ]; then
    echo "[$TIMESTAMP] [OK] cargo check passed (0 errors)" >> "$LOG_FILE"
  fi
  PREV_ERROR_HASH="$CURRENT_ERROR_HASH"

  # 2. Cargo test
  TEST_OUTPUT=$(cd "$MONITOR_DIR" && cargo test --lib --offline 2>&1 | grep "test result")
  TEST_EXIT=$?

  if echo "$TEST_OUTPUT" | grep -q "FAILED"; then
    echo "[$TIMESTAMP] [FAIL] Tests failed:" >> "$LOG_FILE"
    echo "  $TEST_OUTPUT" >> "$LOG_FILE"
  elif [ -n "$TEST_OUTPUT" ]; then
    ONLY_LINE=$(echo "$TEST_OUTPUT" | head -1)
    echo "[$TIMESTAMP] [TEST] $ONLY_LINE" >> "$LOG_FILE"
  fi

  # 3. Warning count
  WARN_COUNT=$(cd "$MONITOR_DIR" && cargo check --lib --offline 2>&1 | grep -c "warning:" 2>/dev/null || echo "0")
  if [ "$WARN_COUNT" != "0" ]; then
    echo "[$TIMESTAMP] [WARN] $WARN_COUNT warnings present" >> "$LOG_FILE"
  fi

  sleep 30
done
