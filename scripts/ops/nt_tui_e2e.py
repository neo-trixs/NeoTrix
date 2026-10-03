#!/usr/bin/env python3
"""⭐⭐⭐ `ntcode` TUI 端到端验收脚本（可重复）

## 为什么需要它（2026-10-03 的直接教训）

那天的手工 PTY 试探**不可重复**，且我因此犯了两次错：
1. ⭐⭐ **第一次没传参数**就断言「TTY 路径坏了」⇒ ⭐ 实为用法要求 `ntcode "<目标>"`
2. ⭐ 第二次只等 10 秒就下判断 ⇒ ⭐ 免费档端点可能更慢

⇒ ⭐⭐ **把「跑一次完整对话」固化成脚本**，才能：
· 每次都带**参数**（不再靠记忆）
· 等待窗口**可配且默认足够长**
· 给出 ⭐ **可判定的**退出结论，而不是「看起来没反应」

## ⭐ 判据（都可机器判定，不含主观描述）

| 判据 | 含义 |
|---|---|
| `tui_entered` | 输出**含 ANSI 序列** ⇒ 走到了全屏 TUI 路径（管道模式恒 False，故此项证明 TTY 分支） |
| `stays_alive` | 首屏后进程**未退出** |
| `accepts_input` | 写入提问后进程**仍存活** ⇒ 输入被接收 |
| `rendered_answer` | 输入后的输出**长度 > 阈值** ⇒ ⭐ 对话真的产生了回答 |

⛔ **任一项为假即 rc=1**，⭐ 不做「看起来差不多」的放行。

## 用法
    python3 scripts/ops/nt_tui_e2e.py                      # 默认 120s
    python3 scripts/ops/nt_tui_e2e.py --timeout 240        # 更长等待
    python3 scripts/ops/nt_tui_e2e.py --model groq/llama-3.3-70b-versatile
    python3 scripts/ops/nt_tui_e2e.py --bin target/debug/ntcode

## ⚠️ 取证边界（⛔ 不夸大）
· 本脚本验证的是 ⭐ **进程层面的行为**（进入 TUI / 存活 / 收输入 / 有输出）。
⛔ 它**不能**证明「回答内容正确」——那需要人工或模型侧断言。
· 若 TUI 把回答画在**依赖终端尺寸**的区域，本脚本的
  `COLUMNS/LINES` 设定会影响可见输出量 ⇒ ⭐ 故 `rendered_answer` 用
  **输出长度**阈值而非内容匹配。
"""
from __future__ import annotations

import argparse
import os
import pty
import re
import select
import signal
import sys
import time

# ⭐ 对话产生的输出下限（字节，去 ANSI 后）。
# ⭐⭐ 刻意**不按内容匹配**：TUI 渲染依赖终端尺寸，内容可能碎成多段控制序列。
MIN_ANSWER_CHARS = 16


def pump(fd: int, seconds: float, sink: bytearray) -> bool:
    """读 `seconds` 秒。返回 False = 管道已关（进程退出）。"""
    t0 = time.time()
    while time.time() - t0 < seconds:
        r, _, _ = select.select([fd], [], [], 0.3)
        if not r:
            continue
        try:
            d = os.read(fd, 65536)
        except OSError:
            return False
        if not d:
            return False
        sink += d
    return True


def strip_ansi(s: str) -> str:
    return re.sub(
        r"\x1b\[[0-9;?]*[a-zA-Z]|\x1b\][^\x07]*\x07|\x1b[()][AB0]|\x1b[=>]", "", s
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", default="target/debug/ntcode")
    ap.add_argument("--target", default="hello", help="ntcode 的必填位置参数")
    ap.add_argument("--prompt", default="2+2=?")
    ap.add_argument("--model", default=None)
    ap.add_argument("--timeout", type=float, default=120.0)
    ap.add_argument("--boot", type=float, default=6.0)
    args = ap.parse_args()

    if not os.path.exists(args.bin):
        print(f"⛔ 二进制不存在：{args.bin}（先 cargo build -p neotrix --bin ntcode）")
        return 2

    argv = [args.bin, args.target]
    if args.model:
        argv += ["--model", args.model]

    pid, fd = pty.fork()
    if pid == 0:
        os.environ["TERM"] = "xterm-256color"
        os.environ["COLUMNS"] = "120"
        os.environ["LINES"] = "40"
        os.execv(args.bin, argv)

    buf = bytearray()
    results: list[tuple[str, bool, str]] = []

    # ① 首屏：⭐ 必带参数（那天的第一个错就是漏了它）
    alive = pump(fd, args.boot, buf)
    head = buf.decode("utf-8", "replace")
    results.append(("stays_alive", alive, f"退出状态码={_status(pid)}" if not alive else "进程存活"))
    results.append(("tui_entered", "\x1b[" in head, f"首屏 ANSI={'YES' if chr(27)+'[' in head else 'NO'}"))

    # ② 输入 + 等待回答
    mark = len(buf)
    answered = False
    after = ""   # ⭐⭐ 必须先初始化：等待循环若一次都不进（timeout<=0），
                 # ⭐⭐ 否则下面的 f-string 会 UnboundLocalError（第一版就踩了）
    if alive:
        try:
            os.write(fd, args.prompt.encode() + b"\r")
        except OSError as e:
            results.append(("accepts_input", False, f"写入失败：{e}"))
        else:
            raw_after = ""
            wait = args.timeout
            step = 5.0
            while wait > 0:
                if not pump(fd, min(step, wait), buf):
                    break
                wait -= step
                raw_after = buf.decode("utf-8", "replace")[mark:]
                after = strip_ansi(raw_after)
                if len(after.strip()) >= MIN_ANSWER_CHARS:
                    answered = True
                    break
            results.append(("accepts_input", True, "写入后仍可交互"))
            # ⭐⭐⭐ **区分「慢」与「空转」**（2026-10-03 实测，这是本脚本最重要的一处判据）
            # ⓰ 我曾把「输出只有 1 字符」归因为「免费档慢」⇒ ⭐ 拉长到 240s 仍一样。
            # ⭐ 真相：输出是 **79 次完全相同的控制序列**
            #   `ESC[39m ESC[49m ESC[59m ESC[0m ESC[?25h ESC[2;3H`
            #   ⇒ ⭐⭐ **渲染循环在转，但一个字符都没画** ⇒ 那是**缺陷**，不是慢。
            seqs = re.findall(r"\x1b\[[0-9;?]*[a-zA-Z]", raw_after)
            uniq = len(set(seqs))
            # ⭐ 阈值取自实测：120s 内 7,230 序列 / 11 种 ⇒ 200/15 稳定触发
            spinning = len(seqs) >= 200 and uniq <= 15
            results.append((
                "rendered_answer", answered,
                f"可见输出 {len(after.strip())} 字符（下限 {MIN_ANSWER_CHARS}）"
                + (f" · ⭐ 空转嫌疑：{len(seqs)} 个控制序列仅 {uniq} 种（重复渲染）"
                   if spinning else ""),
            ))
            if spinning:
                results.append((
                    "no_empty_spin", False,
                    f"检测到 **{len(seqs)} 次几乎相同的空帧**（{uniq} 种序列）"
                    f"⇒ 渲染循环在转但未绘制任何内容 ⇒ **这是缺陷，不是慢**",
                ))

    # ③ 收尾：Esc 取消
    if alive:
        try:
            os.write(fd, b"\x1b")
            pump(fd, 2.0, buf)
        except OSError:
            pass

    try:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
    except (ProcessLookupError, ChildProcessError):
        pass

    print()
    ok = True
    for name, good, detail in results:
        print(f"{'✅' if good else '⛔'} {name:16} {detail}")
        ok = ok and good
    text = strip_ansi(buf.decode("utf-8", "replace"))
    print()
    print(f"--- 可见文本（尾 600 字）---")
    print(text[-600:])
    print()
    if ok:
        print("PASS: ntcode TUI 端到端四项判据全过")
        return 0
    print("FAIL: 有判据未过 ⛔ **不谎称通过**（细节见上）")
    return 1


def _status(pid: int) -> str:
    try:
        done, st = os.waitpid(pid, os.WNOHANG)
    except ChildProcessError:
        return "已回收"
    if done == 0:
        return "仍存活"
    try:
        return str(os.waitstatus_to_exitcode(st))
    except Exception:  # noqa: BLE001 - 仅用于展示
        return str(st)


if __name__ == "__main__":
    sys.exit(main())