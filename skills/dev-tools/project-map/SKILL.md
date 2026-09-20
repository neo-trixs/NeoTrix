# Project Structure Map

## Purpose
项目结构全景视图：快速定位、问题发现、上帝视角

## Trigger Words
- project map
- structure
- panorama
- 项目结构
- 全景视图

## Generated Files

| File | Content |
|------|---------|
| `structure.txt` | 完整目录结构 |
| `modules.txt` | Rust 模块映射 |
| `functions.txt` | 公开函数索引 |

## Usage

```bash
# 生成项目映射
bash skills/dev-tools/project-map/map-project.sh

# 查看结构
cat .project-map/structure.txt

# 查看模块
cat .project-map/modules.txt

# 查看函数
cat .project-map/functions.txt
```

## Integration

Makefile 目标:
```bash
make project-map    # 生成项目映射
make project-view   # 打开项目映射
```
