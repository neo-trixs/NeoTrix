#!/bin/sh
# 汇报器：轮询任务 .done，写 sessions/report-<task>.md + macOS 通知。
# 一律落地项目目录，不落 /tmp（重启会丢）。
ROOT="/Users/neo/Downloads/neotrix"
cd "$ROOT" || exit 9
LOG="$ROOT/sessions/logs"
MARK="$LOG/reporter_reported.txt"
mkdir -p "$LOG"
touch "$MARK"

# task_name:done_file:label
WATCH="v2:$LOG/ftv3.done:v2重训
ftv4:$LOG/ftv4.done:v2重训OOM恢复
lora:$LOG/loraj.done:LoRA-JEV
kev:$LOG/kevdl.done:kev下载
smelt:$LOG/smelt.done:熔炼
ab:$LOG/ab4.done:AB对比
verify:$LOG/verify_queued.done:验证套件
verify2:$LOG/verify2.done:验证套件重跑
verify3:$LOG/verify3.done:验证套件修复后
arch3:$LOG/arch3.done:archive全炼导出
smelt_ingest:$LOG/smelt_ingest.done:熔炼入库
smelt2:$LOG/smelt2.done:熔炼第二轮
asset_ingest:$LOG/asset_ingest.done:素材入库
eval_v2:$LOG/eval_v2.done:v2全量验收
sidecar:$LOG/sidecar.done:sidecar拉起
ab5:$LOG/ab5.done:三方对战
verify4:$LOG/verify4.done:过滤套件v4
cocoons_rebuild:$LOG/cocoons_rebuild.done:cocoons重建
kb_probe:$LOG/kb_probe.done:KB写链二分探针
d1:$LOG/d1.done:D1对照组
selfcall:$LOG/selfcall.done:selfcall全模块
rlcd:$LOG/rlcd.done:RLCD-act-cost
jev_platts:$LOG/jev_platts.done:Jev按域Platt
huangji:$LOG/huangji.done:皇極种子图谱
lora_cal:$LOG/lora_cal.done:LoRA校准版
promote:$LOG/promote.done:LoRA晋升门"

notify() { osascript -e "display notification \"$2\" with title \"NeoTrix\"" >/dev/null 2>&1 || true; }

echo "reporter start $(date '+%H:%M:%S')" >> "$LOG/reporter.log"
while true; do
  echo "$WATCH" | while IFS=: read -r task file label; do
    [ -f "$file" ] || continue
    key="$task:$(cat "$file")"
    grep -qxF "$key" "$MARK" && continue
    echo "$key" >> "$MARK"
    ec=$(sed 's/EXIT://' "$file" | tr -d ' \n')
    if [ "$ec" = "0" ]; then
      {
        echo "# $label — SUCCESS"
        echo "- time: $(date '+%F %T')"
        echo "- exit: 0"
      } > "$ROOT/sessions/report-$task.md"
      notify "$label" "SUCCESS exit:0"
      echo "report $task OK" >> "$LOG/reporter.log"
    else
      {
        echo "# $label — FAILED"
        echo "- time: $(date '+%F %T')"
        echo "- exit: $ec"
        echo "- tail of log:"
        echo '```'
        tail -n 15 "$LOG/$task.log" 2>/dev/null || tail -n 15 "$LOG/${task}3.log" 2>/dev/null
        echo '```'
      } > "$ROOT/sessions/report-$task.md"
      notify "$label" "FAILED exit:$ec"
      echo "report $task FAIL:$ec" >> "$LOG/reporter.log"
    fi
  done
  sleep 300
done
