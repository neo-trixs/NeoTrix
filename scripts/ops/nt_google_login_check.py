#!/usr/bin/env python3
"""Google 登录态只读验证：副本 profile 穿 login 墙否？
只读探针：profile 先拷 temp 副本再起 headless Chrome（不碰用户活体 Chrome，
不写原副本，不输密码）。判读：见账号名/SignOut=登录态在；ServiceLogin/登录=墙。
用法: python3 scripts/ops/nt_google_login_check.py (<=120s)
"""
import json
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

SRC = "/Users/neo/.config/neotrix/nt_browser_profile"
PORT = 9334
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"


def http_get(path, timeout=8):
    with urllib.request.urlopen(f"http://127.0.0.1:{PORT}{path}", timeout=timeout) as r:
        return json.loads(r.read().decode("utf-8"))


def main():
    tmp = tempfile.mkdtemp(prefix="nt_logincheck_")
    prof = f"{tmp}/profile"
    shutil.copytree(SRC, prof, symlinks=True)
    print(f"[login-check] profile copied ({SRC} -> tmp)", flush=True)
    proc = subprocess.Popen(
        [CHROME, "--headless=new", f"--user-data-dir={prof}",
         f"--remote-debugging-port={PORT}", "--remote-allow-origins=*",
         "--no-first-run", "about:blank"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        ver = None
        for _ in range(30):
            try:
                ver = http_get("/json/version")
                break
            except Exception:
                time.sleep(1)
        if ver is None:
            print("[login-check] CDP 起不来（端口无响应）")
            return 2
        ws_url = ver["webSocketDebuggerUrl"]
        import websocket
        ws = websocket.create_connection(ws_url, timeout=15)
        seq = [0]

        def cmd(method, params=None):
            seq[0] += 1
            ws.send(json.dumps({"id": seq[0], "method": method, "params": params or {}}))
            deadline = time.time() + 15
            while time.time() < deadline:
                try:
                    m = json.loads(ws.recv())
                except Exception:
                    continue
                if m.get("id") == seq[0]:
                    return m.get("result", {})
            return {}

        targets = http_get("/json/list")
        page = next((t for t in targets if t.get("type") == "page"), targets[0])
        ws.close()
        ws = websocket.create_connection(page["webSocketDebuggerUrl"], timeout=15)
        cmd("Page.navigate", {"url": "https://accounts.google.com/"})
        time.sleep(6)
        title = cmd("Runtime.evaluate", {"expression": "document.title",
                                         "returnByValue": True})
        body = cmd("Runtime.evaluate", {"expression": "document.body.innerText.slice(0,500)",
                                        "returnByValue": True})
        t = ((title.get("result") or {}).get("value")) or ""
        b = ((body.get("result") or {}).get("value")) or ""
        print(f"[login-check] title={t[:80]}", flush=True)
        print(f"[login-check] body={b[:300]}", flush=True)
        wall = ("ServiceLogin" in b) or ("登录" in t and "管理" not in b)
        import re
        alive = (re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", b) is not None
                 and ("Account" in t or "账号" in b or "Manage" in b or "管理" in b)) \
            or ("SignOut" in b)
        if alive and not wall:
            print("[login-check] VERDICT=LOGGED-IN（登录态在，可附着干活）")
            return 0
        if wall:
            print("[login-check] VERDICT=WALL（登录墙，副本带不过 Keychain 会话）")
            return 1
        print("[login-check] VERDICT=UNKNOWN（换个断言再看）")
        return 3
    finally:
        try:
            proc.terminate()
        except Exception:
            pass
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
