# FluxEnv 后续开发路线图（设计摘要）

**日期**: 2026-03-28  
**状态**: 已确认，进入实现

## 目标（6～8 周）

- **G1**: 重启后 Provider 列表与开关可恢复；密钥仍在系统钥匙串。
- **G2**: 可选项目根、加载 `.env` / `.env.{profile}`、合并预览。
- **G3**: 最小 runner（合并环境 + 工作目录 + 执行命令或打开终端）— 实现阶段再定具体形态。

## 阶段

- **A1**: 本地持久化 Provider 元数据（无密钥明文）+ 启动时 hydrate。
- **B1**: 单项目路径 + profile + `fluxenv-core` 读盘解析 `.env*`。
- **C1**: 最小 `runner` 与文档/README 对齐。

## 边界

- `fluxenv-core`: 纯合并与解析；不碰钥匙串。
- Tauri `secret_store`: 唯一密钥读写。
- Runner 只消费已算好的环境变量表。

## 实现顺序（当前）

1. ✅ 本设计文档  
2. A1: `state_store` + `list_providers` + 变更后自动保存  
3. B1 / C1: 按里程碑迭代
