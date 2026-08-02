import React from "react";
import styles from "./CapabilityHealthPane.module.css";

export interface CapabilityHealthData {
  consciousness_attached: boolean;
  brain_attached: boolean;
  event_bus_attached: boolean;
  evolution_iterations: number;
  tool_grounding_degraded: boolean;
  subagent_results: number;
  context_usage: number;
  provider_resolvable: boolean;
  goals_active: boolean;
  tool_call_count: number;
  turn_count: number;
  session_writable: boolean;
}

const DOMAINS = [
  { key: "NT-CORE", name: "意识核心", icon: "◉", desc: "E8 引导 · GWT 注意力路由 · IIT 集成信息" },
  { key: "NT-MIND", name: "思维", icon: "◈", desc: "推理引擎 · 背景循环 · 元认知" },
  { key: "NT-MEMORY", name: "记忆", icon: "▤", desc: "知识库 · VSA HyperCube · 经验树" },
  { key: "NT-WORLD", name: "世界", icon: "◍", desc: "信息摄取 · OSINT · 抓取" },
  { key: "NT-ACT", name: "行动", icon: "➤", desc: "工具执行 · 进程 · 任务编排" },
  { key: "NT-SHIELD", name: "守护", icon: "◭", desc: "审批门禁 · 权限 · 安全" },
  { key: "NT-IO", name: "界面", icon: "▣", desc: "NeoCodex · 终端 · 预览 · Diff" },
];

export function CapabilityHealthPane({ data }: { data: CapabilityHealthData | null }) {
  if (!data) {
    return (
      <div className={styles.pane} data-testid="capability-health" data-loading="true">
        <div className={styles.title}>能力网健康</div>
        <div className={styles.empty}>正在等待健康报告…</div>
      </div>
    );
  }

  const coreAlive = data.consciousness_attached && data.brain_attached && data.event_bus_attached;
  const ioAlive = data.session_writable && data.provider_resolvable;
  const evolved = data.evolution_iterations > 0;
  const flags = [
    { label: "意识核心已接入", ok: data.consciousness_attached },
    { label: "大脑已接入", ok: data.brain_attached },
    { label: "事件总线已接入", ok: data.event_bus_attached },
    { label: "工具接地正常", ok: !data.tool_grounding_degraded },
    { label: "会话可写", ok: data.session_writable },
    { label: "Provider 可解析", ok: data.provider_resolvable },
    { label: "目标已激活", ok: data.goals_active },
  ];

  // NT-IO is the surface we are viewing through; NT-CORE aggregates the
  // consciousness attachments; the remaining domains derive from the shared
  // substrate (event bus + brain + context).
  const domainStates = DOMAINS.map((d) => {
    if (d.key === "NT-CORE") {
      return { ...d, ok: coreAlive, note: coreAlive ? "三附件就绪" : "存在断链" };
    }
    if (d.key === "NT-IO") {
      return { ...d, ok: ioAlive, note: ioAlive ? "交互链路就绪" : "存在断链" };
    }
    return { ...d, ok: data.event_bus_attached && data.brain_attached, note: data.event_bus_attached ? "链路通畅" : "等待总线" };
  });

  return (
    <div className={styles.pane} data-testid="capability-health" data-loading="false">
      <div className={styles.header}>
        <div className={styles.title}>能力网健康</div>
        <div className={`${styles.badge} ${coreAlive ? styles.badgeOk : styles.badgeWarn}`}>
          {coreAlive ? "核心链路通畅" : "存在断链"}
        </div>
      </div>

      <div className={styles.grid}>
        {domainStates.map((d) => (
          <div key={d.key} className={`${styles.domain} ${d.ok ? styles.domainOk : styles.domainDown}`} data-domain={d.key} data-ok={d.ok}>
            <div className={styles.domainIcon}>{d.icon}</div>
            <div className={styles.domainBody}>
              <div className={styles.domainName}>
                <span>{d.key}</span>
                <span className={d.ok ? styles.dotOk : styles.dotDown} title={d.note} />
              </div>
              <div className={styles.domainDesc}>{d.name} · {d.desc}</div>
              <div className={styles.domainNote}>{d.note}</div>
            </div>
          </div>
        ))}
      </div>

      <div className={styles.flags}>
        {flags.map((f) => (
          <div key={f.label} className={styles.flag} data-ok={f.ok}>
            <span className={f.ok ? styles.flagOk : styles.flagDown}>{f.ok ? "✓" : "✗"}</span>
            <span>{f.label}</span>
          </div>
        ))}
      </div>

      <div className={styles.metrics}>
        <div className={styles.metric}>
          <div className={styles.metricValue}>{data.evolution_iterations}</div>
          <div className={styles.metricLabel}>进化迭代</div>
        </div>
        <div className={styles.metric}>
          <div className={styles.metricValue}>{data.subagent_results}</div>
          <div className={styles.metricLabel}>子代理结果</div>
        </div>
        <div className={styles.metric}>
          <div className={styles.metricValue}>{data.tool_call_count}</div>
          <div className={styles.metricLabel}>工具调用</div>
        </div>
        <div className={styles.metric}>
          <div className={styles.metricValue}>{Math.round((data.context_usage || 0) * 100)}%</div>
          <div className={styles.metricLabel}>上下文占用</div>
        </div>
        <div className={styles.metric}>
          <div className={styles.metricValue}>{data.turn_count}</div>
          <div className={styles.metricLabel}>轮次</div>
        </div>
      </div>

      {evolved && (
        <div className={styles.evolved}>
          <span className={styles.evolvedDot} />
          系统已自我进化 {data.evolution_iterations} 次，能力网持续自愈。
        </div>
      )}
    </div>
  );
}

export default CapabilityHealthPane;
