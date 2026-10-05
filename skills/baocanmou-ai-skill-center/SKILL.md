---
name: baocanmou-ai-skill-center
description: 使用包参谋「方策五环」盘点、中文化、评估、编组和验收 Codex、Claude Code、Hermes、Gemini、Cursor、ZCode、OpenCode、Windsurf、Kimi Code CLI、文心快码 Comate、Qwen Code、TRAE 等 AI 工具的共享技能。当用户提到技能库、中文解释、用途、特点、截图预览、SKILL.md、多 AI 共用、技能链接、审计、冲突、断链或回滚时使用。
---

# 包参谋 AI 技能中心

以 `~/.agents/skills` 作为中心源。先读真实文件，再给中文理解和治理建议；不以排行榜、已安装数量或界面状态代替验收。

## 方策五环

1. **识别**：只读盘点中心源、目标 AI 路径、链接、冲突和 `SKILL.md`。
2. **中文化**：中文主名、明确用途和特点必须齐全并直接显示；英文 ID 与原文保留为兼容层。
3. **评估**：检查结构、依赖、脚本、网络、敏感路径、许可证、重复度和静态风险。
4. **编组**：只在用户授权后，把中心 Skill 连接到指定 AI；不覆盖非托管目录。
5. **验收**：重新扫描真实路径、核对 SHA-256，并在目标 AI 的新会话做最小调用。

## 快速验收

在项目根目录运行：

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- --inspect-summary
python3 tools/skill_center_audit.py
```

## 边界

- App 不自动安装能力情报中的第三方 Skill。
- 不修改第三方 `SKILL.md` 来伪装中文版本；校正内容写入本机独立翻译层。
- 真实截图只作补充；没有真实图片时不制造效果截图，不让装饰信息盖过用途说明。
- 不上传 Skill 内容，不记录凭据、Cookie、私钥或浏览器资料。
- 不删除中心源；断开只移除可验证的受管入口。
- 外部 Skill 的名称、来源、许可和作者必须保真，不能表述为包参谋原创。

路径规则见 [references/兼容路径.md](references/兼容路径.md)，审计证据见 [references/治理规范.md](references/治理规范.md)。
