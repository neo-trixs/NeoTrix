#!/usr/bin/env python3
"""NeoTrix Local Embedding Server (OpenAI-compatible /v1/embeddings).

Serves all-MiniLM-L6-v2 embeddings over HTTP so the Rust KB can generate
real query embeddings without an API key, and without the tokio-blocking
reqwest panic (Rust just calls the local endpoint like any OpenAI API).

Run:
  python3 scripts/kb-embed-server.py            # port 8237
  python3 scripts/kb-embed-server.py --port 9000
Then set:
  NEOTRIX_EMBEDDING_BASE_URL=http://127.0.0.1:8237/v1
  NEOTRIX_EMBEDDING_MODEL=all-MiniLM-L6-v2
  NEOTRIX_EMBEDDING_DIMENSION=384
  NEOTRIX_EMBEDDING_API_KEY=local
"""
import argparse
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MODEL_NAME = "all-MiniLM-L6-v2"
_model = None
_lock = threading.Lock()


def get_model():
    global _model
    if _model is None:
        from sentence_transformers import SentenceTransformer
        with _lock:
            if _model is None:
                _model = SentenceTransformer(MODEL_NAME)
    return _model


class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def _send(self, code, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self._send(200, {"models": [{"id": MODEL_NAME, "dimension": 384}]})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        try:
            req = json.loads(self.rfile.read(length) or b"{}")
        except Exception:
            return self._send(400, {"error": "invalid JSON"})

        model = req.get("model", MODEL_NAME)
        inputs = req.get("input", [])
        if isinstance(inputs, str):
            inputs = [inputs]
        if not isinstance(inputs, list):
            return self._send(400, {"error": "input must be string or list"})
        texts = [str(t) for t in inputs]

        model_obj = get_model()
        try:
            vecs = model_obj.encode(texts, normalize_embeddings=True)
        except Exception as e:
            return self._send(500, {"error": str(e)})

        data = [
            {"object": "embedding", "index": i, "embedding": vec.tolist()}
            for i, vec in enumerate(vecs)
        ]
        self._send(200, {
            "object": "list",
            "model": model,
            "data": data,
            "usage": {"prompt_tokens": 0, "total_tokens": 0},
        })


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8237)
    args = ap.parse_args()
    print(f"NeoTrix Local Embedding Server: http://127.0.0.1:{args.port}/v1/embeddings")
    print(f"  model: {MODEL_NAME} (384 dim, local, no API key)")
    ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
