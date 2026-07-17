# Prompt Git 中文说明

<p align="center">
  <img src="assets/logo.png" alt="Prompt Git 吉祥物：粉紫海马 Logo" width="120" />
</p>

Prompt Git 是面向 AI 开发者的本地桌面端 Prompt 版本管理与效果对比工具。完整英文说明见仓库根目录 [README.md](../README.md)。

## 快速开始

```bash
pnpm install
pnpm tauri dev
```

## 核心能力

1. **版本管理**：提交、历史、Diff、标签、回滚
2. **多模型对比**：并发调用多个模型并打分
3. **批量变量测试**：`{{变量}}` 模板 + CSV / 笛卡尔积
4. **导出**：代码片段、JSON/YAML、Markdown 报告

## 数据目录

所有数据保存在 `~/.prompt-git/`，不会上传云端。

## 模型配置

在 Settings 中添加 OpenAI / DeepSeek / Claude / 通义 / 豆包 / 文心 / Ollama 等配置。API Key 使用 AES-256-GCM 加密，并与系统钥匙串绑定。
