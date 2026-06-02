#!/usr/bin/env python3
"""
TODO 智能同步 + 动态分配系统 - NeoTrix 多 Session 管理
用法：
  python3 scripts/sync_todos.py --smart-sync    # 智能同步（去重+依赖分析+优先级调整）
  python3 scripts/sync_todos.py --allocate        # 动态分配任务给子代理
  python3 scripts/sync_todos.py --status           # 显示所有子代理和任务状态
  python3 scripts/sync_todos.py --auto-run         # 自动运行最高优任务
"""

import argparse
import yaml
import os
import sys
import json
import time
from datetime import datetime, timedelta
from difflib import SequenceMatcher
import re

# 项目根目录
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TODO_MD = os.path.join(PROJECT_ROOT, "TODO.md")
TODO_YML = os.path.join(PROJECT_ROOT, "TODO.yml")
SYNC_RULES = os.path.join(PROJECT_ROOT, "TODO_SYNC.md")
SESSIONS_DIR = os.path.join(PROJECT_ROOT, "sessions", "metadata")
SUBAGENTS_STATE = os.path.join(PROJECT_ROOT, "sessions", "subagents.yml")

# 优先级映射
PRIORITY_MAP = {'high': 3, 'medium': 2, 'low': 1}
PRIORITY_EMOJI = {'high': '🔴', 'medium': '🟡', 'low': '🟢'}
STATUS_EMOJI = {
    'done': '✅',
    'in_progress': '🔄',
    'blocked': '⏳',
    'pending': '⬜',
}


class TodoItem:
    """TODO 项数据结构"""
    def __init__(self, id, title, priority, status, session, session_name,
                 created, updated, subagent=None, files=None, depends_on=None, blocked_by=None,
                 potential_conflict=False):
        self.id = id
        self.title = title
        self.priority = priority  # high, medium, low
        self.status = status  # pending, in_progress, done, blocked
        self.session = session
        self.session_name = session_name
        self.created = created
        self.updated = updated
        self.subagent = subagent
        self.files = files or []
        self.depends_on = depends_on or []
        self.blocked_by = blocked_by or []
        self.potential_conflict = potential_conflict
        self.efficiency_score = 0  # 动态计算
        
    def to_yaml_dict(self):
        """转换为 YAML 字典"""
        return {
            'id': self.id,
            'title': self.title,
            'priority': self.priority,
            'status': self.status,
            'session': self.session,
            'session_name': self.session_name,
            'created': self.created,
            'updated': self.updated,
            'subagent': self.subagent,
            'files': self.files,
            'depends_on': self.depends_on,
            'blocked_by': self.blocked_by,
            'potential_conflict': self.potential_conflict,
        }
    
    def priority_score(self):
        """基础优先级分数"""
        return PRIORITY_MAP.get(self.priority, 0)
    
    def calc_efficiency_score(self, subagents_state):
        """计算效率分数（用于动态排序）"""
        score = self.priority_score() * 10  # 优先级权重最高
        
        # 子代理状态加成
        if self.subagent and self.subagent in subagents_state:
            state = subagents_state[self.subagent]
            if state.get('status') == 'running':
                score += 5  # 正在运行，继续
            elif state.get('status') == 'completed':
                score -= 10  # 已完成，降低优先级
                
        # 等待时间惩罚（等待越久越优先）
        try:
            created_time = datetime.fromisoformat(self.created)
            wait_hours = (datetime.now() - created_time).total_seconds() / 3600
            score += min(wait_hours * 0.5, 5)  # 最多加5分
        except:
            pass
            
        # 依赖惩罚
        if self.depends_on:
            score -= len(self.depends_on) * 3
            
        self.efficiency_score = score
        return score


class SubagentTracker:
    """子代理状态跟踪"""
    def __init__(self):
        self.agents = {}
        self._load()
        
    def _load(self):
        """加载子代理状态"""
        if os.path.exists(SUBAGENTS_STATE):
            try:
                with open(SUBAGENTS_STATE, 'r') as f:
                    self.agents = yaml.safe_load(f) or {}
            except:
                self.agents = {}
    
    def _save(self):
        """保存子代理状态"""
        os.makedirs(os.path.dirname(SUBAGENTS_STATE), exist_ok=True)
        with open(SUBAGENTS_STATE, 'w') as f:
            yaml.dump(self.agents, f, allow_unicode=True, default_flow_style=False)
    
    def register(self, agent_id, task_id, session_id):
        """注册新子代理"""
        self.agents[agent_id] = {
            'task_id': task_id,
            'session_id': session_id,
            'status': 'running',
            'started_at': datetime.now().isoformat(),
            'last_heartbeat': datetime.now().isoformat(),
            'result': None,
        }
        self._save()
        print(f"[AGENT] 注册子代理: {agent_id} -> {task_id}")
    
    def heartbeat(self, agent_id, status=None, result=None):
        """更新子代理心跳"""
        if agent_id in self.agents:
            self.agents[agent_id]['last_heartbeat'] = datetime.now().isoformat()
            if status:
                self.agents[agent_id]['status'] = status
            if result:
                self.agents[agent_id]['result'] = result
            self._save()
    
    def check_stale(self, timeout_minutes=30):
        """检查超时子代理"""
        now = datetime.now()
        stale = []
        for agent_id, info in self.agents.items():
            try:
                last = datetime.fromisoformat(info['last_heartbeat'])
                if (now - last).total_seconds() > timeout_minutes * 60:
                    stale.append(agent_id)
                    info['status'] = 'stale'
            except:
                pass
        if stale:
            self._save()
            print(f"[AGENT] 发现 {len(stale)} 个超时子代理: {stale}")
        return stale
    
    def get_state(self):
        """获取所有子代理状态"""
        return self.agents.copy()
    
    def release(self, agent_id):
        """释放子代理（任务完成）"""
        if agent_id in self.agents:
            self.agents[agent_id]['status'] = 'completed'
            self.agents[agent_id]['completed_at'] = datetime.now().isoformat()
            self._save()
            print(f"[AGENT] 子代理完成: {agent_id}")


class TodoSyncer:
    """智能 TODO 同步器"""
    def __init__(self):
        self.items = []
        self.conflicts = []
        self.subagent_tracker = SubagentTracker()
        
    def load_todo_md(self):
        """从 TODO.md 加载现有 TODO 项（增量更新，不丢失数据）"""
        if not os.path.exists(TODO_MD):
            print(f"[WARN] {TODO_MD} 不存在，将创建新文件")
            return
        
        with open(TODO_MD, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 解析 Markdown（支持多种格式）
        # 匹配：### emoji ID: Title
        pattern = r'###\s+(\S*)\s+(S-\w+):\s*(.+?)(?=\n###|\n##|\Z)'
        matches = re.findall(pattern, content, re.DOTALL)
        
        for match in matches:
            emoji_status = match[0].strip()
            todo_id = match[1].strip()
            body = match[2]
            
            # 提取状态
            status = 'pending'
            if '✅' in emoji_status or 'done' in body.lower():
                status = 'done'
            elif '🔄' in emoji_status or 'in_progress' in body.lower():
                status = 'in_progress'
            elif '⏳' in emoji_status or 'blocked' in body.lower():
                status = 'blocked'
            
            # 提取优先级（从所在段落）
            priority = 'medium'
            if '🔴' in body or '高优先级' in body:
                priority = 'high'
            elif '🟡' in body or '中优先级' in body:
                priority = 'medium'
            elif '🟢' in body or '低优先级' in body:
                priority = 'low'
            
            # 提取详细信息
            session = ""
            session_match = re.search(r'\*\*Session\*\*:\s*(\S+)', body)
            if session_match:
                session = session_match.group(1)
            
            subagent = None
            subagent_match = re.search(r'\*\*子代理\*\*:\s*(\S+)', body)
            if subagent_match:
                subagent = subagent_match.group(1)
            
            files = []
            files_match = re.search(r'\*\*文件\*\*:\s*(.+?)(?=\n\*\*)', body, re.DOTALL)
            if files_match:
                files = [f.strip() for f in files_match.group(1).split(',')]
            
            # 检查是否已存在（增量更新）
            existing = next((item for item in self.items if item.id == todo_id), None)
            if existing:
                # 更新现有项
                existing.status = status
                existing.updated = datetime.now().isoformat()
                if subagent:
                    existing.subagent = subagent
                print(f"[INFO] 更新 TODO: {todo_id}")
            else:
                # 新增项
                item = TodoItem(
                    id=todo_id,
                    title=body.split('\n')[0].strip(),
                    priority=priority,
                    status=status,
                    session=session,
                    session_name=f"{todo_id} {body.split(chr(10))[0].strip()}",
                    created=datetime.now().isoformat(),
                    updated=datetime.now().isoformat(),
                    subagent=subagent,
                    files=files,
                )
                self.items.append(item)
                print(f"[INFO] 添加 TODO: {todo_id} - {item.title[:30]}")
        
        print(f"[INFO] 从 {TODO_MD} 加载了 {len(self.items)} 个 TODO 项")
    
    def smart_analyze(self):
        """智能分析：去重 + 依赖检查 + 冲突检测"""
        print(f"[SMART] 开始智能分析...")
        
        # 1. 去重分析
        to_remove = []
        for i, item1 in enumerate(self.items):
            for j, item2 in enumerate(self.items):
                if i >= j:
                    continue
                
                # ID 完全相同
                if item1.id == item2.id:
                    print(f"[DUPLICATE] ID 重复: {item1.id}")
                    # 合并：保留更新时间更新的
                    if item1.updated > item2.updated:
                        to_remove.append(j)
                    else:
                        to_remove.append(i)
                    continue
                
                # 标题相似度 > 0.8
                ratio = SequenceMatcher(None, item1.title, item2.title).ratio()
                if ratio > 0.8:
                    self.conflicts.append({
                        'type': 'similar_title',
                        'id1': item1.id,
                        'id2': item2.id,
                        'similarity': ratio,
                    })
                    print(f"[CONFLICT] 标题相似度 {ratio:.2f}: {item1.id} vs {item2.id}")
        
        # 移除重复项（从后往前删，避免索引变化）
        for idx in sorted(set(to_remove), reverse=True):
            if idx < len(self.items):
                removed = self.items.pop(idx)
                print(f"[INFO] 移除重复项: {removed.id}")
        
        # 2. 依赖关系检查
        for item in self.items:
            if item.depends_on:
                for dep_id in item.depends_on:
                    dep_item = next((i for i in self.items if i.id == dep_id), None)
                    if not dep_item:
                        print(f"[WARN] {item.id} 依赖的 {dep_id} 不存在")
                    elif dep_item.status != 'done':
                        item.status = 'blocked'
                        item.blocked_by.append(dep_id)
                        print(f"[BLOCKED] {item.id} 被阻塞，等待 {dep_id}")
        
        # 3. 子代理状态同步
        stale_agents = self.subagent_tracker.check_stale()
        for agent_id in stale_agents:
            # 找到该子代理对应的任务，标记为异常
            for item in self.items:
                if item.subagent == agent_id:
                    item.status = 'blocked'
                    item.potential_conflict = True
                    print(f"[STALE] 子代理超时: {agent_id} -> {item.id}")
        
        print(f"[SMART] 分析完成: {len(self.items)} 个 TODO, {len(self.conflicts)} 个冲突")
    
    def dynamic_priority_adjustment(self):
        """动态优先级调整（基于效率分数）"""
        print(f"[DYNAMIC] 动态优先级调整...")
        
        subagents_state = self.subagent_tracker.get_state()
        
        for item in self.items:
            item.calc_efficiency_score(subagents_state)
        
        # 按效率分数降序排序
        self.items.sort(key=lambda x: -x.efficiency_score)
        
        print(f"[DYNAMIC] 已完成优先级调整")
    
    def allocate_tasks(self, max_parallel=3):
        """动态分配任务给子代理"""
        print(f"[ALLOCATE] 开始任务分配 (max_parallel={max_parallel})...")
        
        # 检查当前运行的子代理数
        running = [a for a in self.subagent_tracker.get_state().values() 
                    if a.get('status') == 'running']
        
        if len(running) >= max_parallel:
            print(f"[ALLOCATE] 已达最大并行数 ({max_parallel})，等待...")
            return []
        
        # 找到可分配的任务（pending + 未阻塞）
        available = [item for item in self.items 
                       if item.status == 'pending' and not item.blocked_by]
        
        if not available:
            print(f"[ALLOCATE] 无可用任务")
            return []
        
        # 分配前 N 个（根据效率分数）
        to_allocate = available[:max_parallel - len(running)]
        
        allocations = []
        for item in to_allocate:
            # 生成子代理 ID
            agent_id = f"ses_{datetime.now().strftime('%Y%m%d%H%M%S')}"
            
            # 更新任务状态
            item.status = 'in_progress'
            item.subagent = agent_id
            item.updated = datetime.now().isoformat()
            
            allocations.append({
                'agent_id': agent_id,
                'task_id': item.id,
                'title': item.title,
                'priority': item.priority,
            })
            
            # 注册子代理
            self.subagent_tracker.register(agent_id, item.id, item.session)
        
        if allocations:
            self.save_todo_md()
            self.save_todo_yml()
        
        print(f"[ALLOCATE] 分配了 {len(allocations)} 个任务")
        return allocations
    
    def save_todo_md(self):
        """保存为 TODO.md（保持人类可读 + 状态更新）"""
        with open(TODO_MD, 'w', encoding='utf-8') as f:
            f.write("# NeoTrix TODO 列表\n")
            f.write("> 智能同步生成，最后更新：" + datetime.now().isoformat() + "\n")
            f.write("> 代数视角：Agent 操作向量空间 V，变换矩阵 T\n\n")
            
            # 按优先级分组
            for priority, emoji in [('high', '🔴'), ('medium', '🟡'), ('low', '🟢')]:
                items = [i for i in self.items if i.priority == priority]
                if not items:
                    continue
                
                f.write(f"## {emoji} {priority.capitalize()} 优先级\n\n")
                
                for item in items:
                    status_emoji = STATUS_EMOJI.get(item.status, '⬜')
                    
                    f.write(f"### {status_emoji} {item.id}: {item.title}\n\n")
                    f.write(f"**状态**: {item.status}  \n")
                    f.write(f"**Session**: {item.session} ({item.session_name})\n")
                    if item.subagent:
                        f.write(f"**子代理**: {item.subagent}\n")
                    if item.files:
                        f.write(f"**文件**: {', '.join(item.files)}\n")
                    f.write(f"**更新**: {item.updated}\n")
                    f.write(f"**效率分数**: {item.efficiency_score:.1f}\n\n")
            
            # 冲突报告
            if self.conflicts:
                f.write("## ⚠️ 冲突报告\n\n")
                for conflict in self.conflicts:
                    f.write(f"- {conflict['type']}: {conflict.get('id1', '')} vs {conflict.get('id2', '')}")
                    if 'similarity' in conflict:
                        f.write(f" (相似度: {conflict['similarity']:.2f})")
                    f.write("\n")
        
        print(f"[INFO] 已保存: {TODO_MD}")
    
    def save_todo_yml(self):
        """保存为 TODO.yml（机器可读格式）"""
        data = {
            'meta': {
                'generated_at': datetime.now().isoformat(),
                'total_items': len(self.items),
                'conflicts': len(self.conflicts),
            },
            'items': [item.to_yaml_dict() for item in self.items],
            'conflicts': self.conflicts,
            'subagents': self.subagent_tracker.get_state(),
        }
        
        with open(TODO_YML, 'w', encoding='utf-8') as f:
            yaml.dump(data, f, allow_unicode=True, default_flow_style=False)
        
        print(f"[INFO] 已保存: {TODO_YML}")
    
    def print_status(self):
        """打印所有任务和子代理状态"""
        print("\n" + "="*60)
        print("NeoTrix TODO 状态报告")
        print("="*60)
        
        # 子代理状态
        print("\n[子代理状态]")
        agents = self.subagent_tracker.get_state()
        if agents:
            for agent_id, info in agents.items():
                print(f"  {agent_id}: {info.get('status')} -> {info.get('task_id')}")
        else:
            print("  无活动子代理")
        
        # 任务统计
        print("\n[任务统计]")
        stats = {}
        for item in self.items:
            key = f"{item.priority}_{item.status}"
            stats[key] = stats.get(key, 0) + 1
        
        for key, count in sorted(stats.items()):
            print(f"  {key}: {count}")
        
        # 最高效任务（前5个）
        print("\n[最高效任务 TOP 5]")
        sorted_items = sorted(self.items, key=lambda x: -x.efficiency_score)
        for item in sorted_items[:5]:
            print(f"  {item.id}: 分数={item.efficiency_score:.1f}, 状态={item.status}")


def watch_files(root_dir: str, callback):
    """监控 TODO.md 和 .rs 文件变更，触发同步"""
    try:
        from watchdog.observers import Observer
        from watchdog.events import FileSystemEventHandler

        class TodoHandler(FileSystemEventHandler):
            def on_modified(self, event):
                if event.is_directory:
                    return
                path = event.src_path
                if path.endswith('TODO.md') or path.endswith('.rs'):
                    callback(path)

        event_handler = TodoHandler()
        observer = Observer()
        observer.schedule(event_handler, root_dir, recursive=True)
        print(f"[watch] 监控目录: {root_dir}")
        print("[watch] 按 Ctrl+C 停止")
        observer.start()
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            observer.stop()
        observer.join()
    except ImportError:
        # fallback: 基于轮询的简单监控
        print("[watch] watchdog 未安装，使用轮询模式 (pip install watchdog 可加速)")
        last_mtimes = {}
        while True:
            for dirpath, _, filenames in os.walk(root_dir):
                for f in filenames:
                    if not (f.endswith('TODO.md') or f.endswith('.rs')):
                        continue
                    fp = os.path.join(dirpath, f)
                    try:
                        mtime = os.path.getmtime(fp)
                        if fp in last_mtimes and last_mtimes[fp] != mtime:
                            callback(fp)
                        last_mtimes[fp] = mtime
                    except OSError:
                        pass
            time.sleep(2)


def main():
    parser = argparse.ArgumentParser(description='NeoTrix 智能 TODO 同步 + 动态分配')
    parser.add_argument('--smart-sync', action='store_true', help='智能同步（去重+依赖分析+优先级调整）')
    parser.add_argument('--allocate', action='store_true', help='动态分配任务给子代理')
    parser.add_argument('--status', action='store_true', help='显示所有子代理和任务状态')
    parser.add_argument('--auto-run', action='store_true', help='自动运行最高优任务')
    parser.add_argument('--watch', action='store_true', help='监控文件变更自动同步')
    parser.add_argument('--max-parallel', type=int, default=3, help='最大并行子代理数')
    args = parser.parse_args()
    
    syncer = TodoSyncer()
    syncer.load_todo_md()
    
    def on_change(path):
        print(f"\n[watch] 变更: {os.path.basename(path)}")
        try:
            syncer.load_todo_md()
            syncer.smart_analyze()
            syncer.dynamic_priority_adjustment()
            syncer.save_todo_md()
            syncer.save_todo_yml()
            print(f"[watch] 同步完成")
        except Exception as e:
            print(f"[watch] 同步失败: {e}")
    
    if args.watch:
        watch_files(PROJECT_ROOT, on_change)
        return
    
    if args.smart_sync or args.allocate or args.auto_run:
        syncer.smart_analyze()
        syncer.dynamic_priority_adjustment()
        syncer.save_todo_md()
        syncer.save_todo_yml()
        
    if args.allocate or args.auto_run:
        allocations = syncer.allocate_tasks(max_parallel=args.max_parallel)
        if allocations:
            print("\n[分配结果]")
            for alloc in allocations:
                print(f"  {alloc['agent_id']} -> {alloc['task_id']} ({alloc['title'][:30]})")
            print("\n提示: 使用以下命令启动子代理：")
            for alloc in allocations:
                print(f"  Task tool: {alloc['task_id']}")
                
    if args.status or args.smart_sync:
        syncer.print_status()
    
    if not any([args.smart_sync, args.allocate, args.status, args.auto_run, args.watch]):
        parser.print_help()


if __name__ == '__main__':
    main()
