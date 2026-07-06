# Playwright 集成使用说明

## 概述

NeoTrix 现在支持可选的 Playwright 集成，用于真实的浏览器自动化验证。

## 启用方式

在 `Cargo.toml` 中启用 `playwright` feature：

```bash
cargo build --features playwright
```

或者在 `Cargo.toml` 中添加：

```toml
[features]
default = []
playwright = ["shellexpand"]
```

## 安装 Playwright

启用 `playwright` feature 后，需要安装 Playwright CLI：

```bash
npm install -g playwright
npx playwright install chromium firefox webkit
```

或者仅在项目中安装：

```bash
cd /path/to/neotrix
npm init -y
npm install playwright
npx playwright install chromium
```

## 配置

创建 `playwright.json` 配置文件（可选）：

```json
{
  "enabled": true,
  "browser": "chromium",
  "headless": true,
  "screenshot_dir": "/tmp/neotrix_screenshots",
  "timeout_ms": 30000,
  "playwright_cli_path": null
}
```

配置加载优先级：
1. `/etc/neotrix/playwright.json`
2. `~/.neotrix/playwright.json`
3. `./playwright.json`
4. 环境变量
5. 默认值

### 环境变量

- `NEOTRIX_PLAYWRIGHT_ENABLED`: 是否启用（true/false）
- `NEOTRIX_PLAYWRIGHT_BROWSER`: 浏览器类型（chromium/firefox/webkit）
- `NEOTRIX_PLAYWRIGHT_HEADLESS`: 是否 headless 模式（true/false）
- `NEOTRIX_PLAYWRIGHT_SCREENSHOT_DIR`: 截图保存目录
- `NEOTRIX_PLAYWRIGHT_TIMEOUT_MS`: 超时时间（毫秒）
- `NEOTRIX_PLAYWRIGHT_CLI_PATH`: Playwright CLI 路径（可选）

## 功能

### 1. playwright_verify

验证目标 URL，生成截图并评估可访问性。

**MCP 工具调用**：
```json
{
  "name": "playwright_verify",
  "arguments": {
    "target": "https://example.com"
  }
}
```

**返回**（真实模式）：
```json
{
  "target": "https://example.com",
  "status": "verified",
  "screenshot": "/tmp/neotrix_screenshots/screenshot_1234567890.png",
  "page_title": "Example Domain",
  "final_url": "https://example.com/",
  "accessibility_score": 0.95,
  "timestamp": "2026-04-29T12:00:00Z",
  "playwright_real": true
}
```

### 2. cua_check

检查页面的 CUA（Computer-Using Agent）可访问性。

**MCP 工具调用**：
```json
{
  "name": "cua_check",
  "arguments": {
    "check": "https://example.com"
  }
}
```

**返回**（真实模式）：
```json
{
  "check_target": "https://example.com",
  "cua_compatible": true,
  "accessibility_issues": [],
  "score": 0.92,
  "interactive_elements_count": 5,
  "cua_real": true
}
```

## 优雅降级

如果 Playwright 不可用（未安装或配置错误），工具会自动降级到模拟模式：

```json
{
  "target": "https://example.com",
  "status": "verified",
  "playwright_real": false,
  "warning": "Using mock Playwright - enable 'playwright' feature and install Playwright for real verification"
}
```

## 测试

运行测试（模拟模式）：
```bash
cargo test --lib mcp_tools
```

运行测试（真实模式，需要安装 Playwright）：
```bash
cargo test --lib --features playwright mcp_tools
```

## 依赖变更

- **新增可选依赖**：`shellexpand = "3"`（用于配置文件路径展开）
- **不使用** `playwright-rs` crate（避免编译时环境变量问题）

## 实现细节

- 使用 subprocess 调用 Playwright CLI（`npx playwright`）
- 动态生成 Node.js 脚本执行浏览器操作
- 截图保存在配置指定的目录（默认 `/tmp/neotrix_screenshots`）
- 支持 chromium、firefox、webkit 三种浏览器

## 故障排查

### Playwright CLI 未找到

```
Error: Playwright CLI not found. Install with: npm install -g playwright
```

**解决**：按照上面的"安装 Playwright"步骤操作。

### 截图目录不存在

```
Error: Failed to create screenshot dir: Permission denied
```

**解决**：修改 `playwright.json` 中的 `screenshot_dir` 为可写目录。

### Node.js 脚本执行失败

检查 Playwright 是否正确安装：
```bash
npx playwright --version
```
