// ══════════════════════════════════════════════════════════════════════════
//  Icon — 统一图标组件（macOS 极简风格）
//  基于 Lucide 图标库，统一尺寸/颜色/ strokeWidth
// ══════════════════════════════════════════════════════════════════════════
import { type JSX } from 'solid-js'
import { clsx } from 'clsx'

// ── 图标映射 ──
// 所有图标统一从 lucide-solid 导入，保持一致性

import {
  // 因果链节点
  MessageSquare,    // input
  Search,           // analyze
  BookOpen,         // search
  Zap,              // execute
  Check,            // output
  Lightbulb,        // decision
  Shield,           // approval
  X,                // error
  Paperclip,        // context

  // 通用操作
  Copy,
  Pencil,
  RotateCcw,
  Trash2,
  Send,
  Square,
  Loader2,
  ChevronDown,
  ChevronRight,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  RefreshCw,
  ExternalLink,
  Settings,
  Plus,
  Minus,
  XCircle,
  Eye,
  EyeOff,
  Terminal,
  Folder,
  FolderOpen,
  File,
  FileText,
  FileCode2,
  Image,
  Table,
  Globe,
  Brain,
  Activity,
  Wallet,
  Coins,
  Database,
  Layers,
  LayoutDashboard,
  GitBranch,
  GitCommitHorizontal,
  Upload,
  Download,
  Plug,
  Cpu,
  Mic,
  MicOff,
  Volume2,
  VolumeX,
  Circle,
  Pause,
  Play,
  Square as SquareIcon,
  MoreVertical,
  Pin,
  History,
  Clock,
  CalendarClock,
  ListTodo,
  CheckSquare,
  MousePointer2,
  Type,
  Undo2,
  Redo2,
  MonitorPlay,
  Sparkles,
  Briefcase,
  Code2,
  ArrowLeft,
  ArrowDownToLine,
  Filter,
  Maximize2,
  Minimize2,
  ZoomIn,
  ZoomOut,
  RotateCw,
  Wrench,
  Box,
  Store,
  Puzzle,
  Power,
  PowerOff,
  ListTree,
  Tag,
  Star,
  BarChart3,
  TrendingUp,
  Info,
  Workflow,
  Loader,
  Database as DatabaseIcon,
} from 'lucide-solid'

// ── 图标名称类型 ──

export type IconName =
  // 因果链
  | 'input' | 'analyze' | 'search' | 'execute' | 'output'
  | 'decision' | 'approval' | 'error' | 'context'
  // 通用
  | 'copy' | 'edit' | 'retry' | 'delete' | 'send' | 'stop'
  | 'loading' | 'chevron-down' | 'chevron-right'
  | 'check' | 'check-circle' | 'alert' | 'alert-circle'
  | 'refresh' | 'external' | 'settings' | 'plus' | 'minus'
  | 'eye' | 'eye-off' | 'terminal' | 'folder' | 'folder-open'
  | 'file' | 'file-text' | 'file-code' | 'image' | 'table'
  | 'globe' | 'brain' | 'activity' | 'wallet' | 'coins'
  | 'database' | 'layers' | 'layout' | 'git-branch' | 'git-commit'
  | 'upload' | 'download' | 'plug' | 'cpu' | 'mic' | 'mic-off'
  | 'volume' | 'volume-off' | 'circle' | 'pause' | 'play' | 'square'
  | 'more' | 'pin' | 'history' | 'clock' | 'calendar' | 'list'
  | 'check-square' | 'mouse' | 'type' | 'undo' | 'redo'
  | 'monitor' | 'sparkles' | 'briefcase' | 'code' | 'arrow-left'
  | 'filter' | 'maximize' | 'minimize' | 'zoom-in' | 'zoom-out'
  | 'rotate' | 'wrench' | 'box' | 'store' | 'puzzle' | 'power'
  | 'list-tree' | 'tag' | 'star' | 'bar-chart' | 'trending'
  | 'info' | 'workflow' | 'loader'

// ── 图标组件 ──

interface IconProps {
  name: IconName
  size?: number
  color?: string
  class?: string
  strokeWidth?: number
}

const ICON_MAP: Record<IconName, typeof MessageSquare> = {
  'input': MessageSquare,
  'analyze': Search,
  'search': BookOpen,
  'execute': Zap,
  'output': Check,
  'decision': Lightbulb,
  'approval': Shield,
  'error': X,
  'context': Paperclip,
  'copy': Copy,
  'edit': Pencil,
  'retry': RotateCcw,
  'delete': Trash2,
  'send': Send,
  'stop': Square,
  'loading': Loader2,
  'chevron-down': ChevronDown,
  'chevron-right': ChevronRight,
  'check': Check,
  'check-circle': CheckCircle,
  'alert': AlertTriangle,
  'alert-circle': AlertCircle,
  'refresh': RefreshCw,
  'external': ExternalLink,
  'settings': Settings,
  'plus': Plus,
  'minus': Minus,
  'eye': Eye,
  'eye-off': EyeOff,
  'terminal': Terminal,
  'folder': Folder,
  'folder-open': FolderOpen,
  'file': File,
  'file-text': FileText,
  'file-code': FileCode2,
  'image': Image,
  'table': Table,
  'globe': Globe,
  'brain': Brain,
  'activity': Activity,
  'wallet': Wallet,
  'coins': Coins,
  'database': Database,
  'layers': Layers,
  'layout': LayoutDashboard,
  'git-branch': GitBranch,
  'git-commit': GitCommitHorizontal,
  'upload': Upload,
  'download': Download,
  'plug': Plug,
  'cpu': Cpu,
  'mic': Mic,
  'mic-off': MicOff,
  'volume': Volume2,
  'volume-off': VolumeX,
  'circle': Circle,
  'pause': Pause,
  'play': Play,
  'square': SquareIcon,
  'more': MoreVertical,
  'pin': Pin,
  'history': History,
  'clock': Clock,
  'calendar': CalendarClock,
  'list': ListTodo,
  'check-square': CheckSquare,
  'mouse': MousePointer2,
  'type': Type,
  'undo': Undo2,
  'redo': Redo2,
  'monitor': MonitorPlay,
  'sparkles': Sparkles,
  'briefcase': Briefcase,
  'code': Code2,
  'arrow-left': ArrowLeft,
  'filter': Filter,
  'maximize': Maximize2,
  'minimize': Minimize2,
  'zoom-in': ZoomIn,
  'zoom-out': ZoomOut,
  'rotate': RotateCw,
  'wrench': Wrench,
  'box': Box,
  'store': Store,
  'puzzle': Puzzle,
  'power': Power,
  'list-tree': ListTree,
  'tag': Tag,
  'star': Star,
  'bar-chart': BarChart3,
  'trending': TrendingUp,
  'info': Info,
  'workflow': Workflow,
  'loader': Loader,
}

export function Icon(props: IconProps) {
  const LucideIcon = ICON_MAP[props.name]
  return (
    <LucideIcon
      size={props.size ?? 16}
      color={props.color}
      stroke-width={props.strokeWidth ?? 1.5}
      class={props.class}
    />
  )
}
