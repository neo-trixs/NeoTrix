# NeoTrix Conversation App — Full Rebuild Plan

## Current State Analysis
- Frontend: Monolith HTML+CSS+JS inline (~600 lines)
- Backend: Tauri with 12 domain plugins (chat, session, agent, etc.)
- Working: Session management, basic chat, provider config, model selection
- Missing: Streaming, agent loop, memory system, proper state management

## Osaurus Patterns to Absorb

### 1. ChatSession State Management
- Turn-based history (`turns: [ChatTurn]`)
- Streaming state (`isStreaming: Bool`)
- Model selection (`selectedModel: String?`)
- Agent loop state (`currentTodo: AgentTodo?`)
- Memory system (`identity`, `pinned`, `episodes`)
- Prompt queue (`promptQueue: PromptQueue`)

### 2. Message Rendering
- NativeMessageCellView pattern
- Markdown rendering
- Code blocks with highlighting
- Thinking blocks
- Tool call visualization
- Follow-up suggestions

### 3. Composer
- FloatingInputCard pattern
- Attachment support
- Model picker
- Slash commands
- Queued sends

### 4. Agent Loop
- Todo list management
- Tool call execution
- Completion summary
- Clarification requests

### 5. Streaming Engine
- VisibleBlocksStore pattern
- Block memoization
- Real-time updates

## Implementation Plan

### Phase 1: Component Architecture (Current)
Create modular TypeScript components:
- `src/types.ts` - Type definitions
- `src/state.ts` - Central state management
- `src/components/` - UI components
- `src/lib/` - Utilities and API

### Phase 2: State Management
Implement reactive state:
- ChatSession state
- Turn management
- Model selection
- Streaming state

### Phase 3: Message Rendering
Build message components:
- User messages
- Assistant messages
- Thinking blocks
- Tool calls
- Code blocks

### Phase 4: Composer
Implement input system:
- Text input
- Attachments
- Model picker
- Slash commands

### Phase 5: Streaming
Add real-time support:
- Backend SSE/WebSocket
- Frontend listeners
- Block updates

### Phase 6: Agent Loop
Implement agent features:
- Todo lists
- Tool visualization
- Completion summaries

### Phase 7: Memory System
Add memory features:
- Identity layer
- Pinned facts
- Episode recording

## File Structure

```
neocodex-frontend/src/
├── types.ts              # Type definitions
├── state.ts              # State management
├── api.ts                # API layer
├── components/
│   ├── Sidebar.ts        # Session sidebar
│   ├── MessageThread.ts  # Message list
│   ├── MessageBubble.ts  # Individual message
│   ├── Composer.ts       # Input area
│   ├── ModelPicker.ts    # Model selection
│   ├── ToolCall.ts       # Tool visualization
│   ├── ThinkingBlock.ts  # Thinking display
│   └── FollowUp.ts       # Follow-up suggestions
├── lib/
│   ├── markdown.ts       # Markdown renderer
│   ├── code.ts           # Code highlighting
│   └── streaming.ts      # Streaming handler
└── main.ts               # Entry point
```

## Key Design Decisions

1. **Reactive State** - Use simple event emitter pattern
2. **Component-based** - Replace monolith with modules
3. **Streaming-first** - All responses stream by default
4. **Memory-budgeted** - Inject ~800 tokens max per request
5. **Background distillation** - Don't block UI for memory ops

## Implementation Priority

### Critical (Must Have)
1. Component architecture
2. State management
3. Message rendering
4. Basic streaming
5. Session persistence

### Important (Should Have)
1. Markdown rendering
2. Code blocks
3. Attachment support
4. Model picker
5. Agent loop basics

### Nice to Have
1. Thinking blocks
2. Follow-up suggestions
3. Privacy filter
4. Memory system
5. Advanced tool visualization
