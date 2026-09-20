#!/bin/bash
MODEL="/Users/neo/Downloads/neotrix/models/Spark-X2.5-4B-abliterated-FIT-BALANCED-2.68GiB-Q5_K_S.gguf"
LLAMA="/tmp/llama.cpp-spark/build/bin/llama-server"
PORT=${1:-8080}
if curl -s http://127.0.0.1:$PORT/health > /dev/null 2>&1; then
    echo "✅ Spark-X2.5 already running on :$PORT"
    exit 0
fi
exec "$LLAMA" -m "$MODEL" --host 127.0.0.1 --port $PORT \
  -ngl 99 -c 131072 -np 1 --jinja "$@"
