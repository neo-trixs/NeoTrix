# Pre-Build Check

## Purpose
构建前环境检查：依赖、配置、状态验证

## Trigger Words
- pre-build check
- build check
- 构建前检查
- environment check

## Workflow

### Run Pre-Build Check
```bash
scripts/pre-build-check.sh
```

### Checks Performed
1. Cargo 工具链版本
2. Node.js/npm 版本
3. Tauri CLI 版本
4. 系统依赖（webkit2gtk, etc.）
5. 环境变量配置
6. Git 状态（未提交更改）

## Makefile Integration
```makefile
desktop-check:
	@scripts/pre-build-check.sh
	@scripts/build-desktop.sh check
```

## Error Handling
- 非阻塞警告：未提交更改
- 阻塞错误：缺少依赖
- 退出码：0=通过, 1=失败
