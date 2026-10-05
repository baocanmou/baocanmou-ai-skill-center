#!/usr/bin/env python3
"""包参谋 AI 技能中心只读审计器。

盘点共享技能源及各 AI 工具入口，不会新建、删除、修复或替换任何文件。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Iterable


TOOL_PATHS = {
    "Codex": ".codex/skills",
    "Claude Code": ".claude/skills",
    "Hermes": ".hermes/skills",
    "Gemini CLI": ".gemini/skills",
    "Cursor": ".cursor/skills",
    "ZCode": ".zcode/skills",
    "OpenCode": ".config/opencode/skills",
    "Windsurf": ".windsurf/skills",
    # Kimi Code CLI 本身读取 ~/.agents/skills；这里只用于发现它自有目录里的同名重复入口。
    "Kimi Code CLI": ".kimi-code/skills",
    "文心快码 Comate": ".comate/skills",
    "Qwen Code": ".qwen/skills",
    "TRAE": ".trae/skills",
    "TRAE CN": ".trae-cn/skills",
}
FRONTMATTER = re.compile(r"\A---\s*\n(?P<body>.*?)\n---(?:\s*\n|\Z)", re.DOTALL)
FIELD = re.compile(r"^(?P<key>[A-Za-z0-9_-]+):\s*(?P<value>.*)$")


@dataclass
class SkillResult:
    name: str
    path: str
    declared_name: str | None
    description: str | None
    file_count: int
    byte_count: int
    sha256: str
    git_origin: str | None
    git_revision: str | None
    issues: list[str]
    kind: str
    warnings: list[str]
    entrypoint_chars: int


@dataclass
class ToolResult:
    name: str
    path: str
    exists: bool
    entry_count: int
    symlink_count: int
    broken_symlink_count: int
    central_link_count: int
    root_skill_count: int = 0
    local_directory_count: int = 0
    same_name_local_count: int = 0
    divergent_local_skills: list[dict[str, object]] = field(default_factory=list)
    missing_shared_names: list[str] = field(default_factory=list)


def parse_frontmatter(text: str) -> dict[str, str]:
    match = FRONTMATTER.search(text)
    if not match:
        return {}
    fields: dict[str, str] = {}
    lines = match.group("body").splitlines()
    for index, line in enumerate(lines):
        found = FIELD.match(line)
        if not found:
            continue
        value = found.group("value").strip()
        if value in {"|", "|-", "|+", ">", ">-", ">+"}:
            chunks = []
            for following in lines[index + 1:]:
                if following.strip() and not following.startswith((" ", "\t")):
                    break
                chunks.append(following.strip())
            value = (" " if value.startswith(">") else "\n").join(chunks).strip()
        elif value.startswith('"'):
            try:
                value = json.loads(value)
            except json.JSONDecodeError:
                value = value.strip('"')
        elif value.startswith("'") and value.endswith("'"):
            value = value[1:-1].replace("''", "'")
        fields[found.group("key")] = value
    return fields


def visible_files(root: Path) -> Iterable[Path]:
    for current, dirnames, filenames in os.walk(root, followlinks=False):
        dirnames[:] = sorted(
            name for name in dirnames if name not in {".git", "node_modules", "__pycache__"}
        )
        for filename in sorted(filenames):
            path = Path(current) / filename
            if not path.is_symlink() and path.is_file():
                yield path


def digest_tree(root: Path) -> tuple[int, int, str]:
    digest = hashlib.sha256()
    files = list(visible_files(root))
    total = 0
    for path in files:
        relative = path.relative_to(root).as_posix().encode("utf-8")
        content = path.read_bytes()
        total += len(content)
        digest.update(len(relative).to_bytes(4, "big"))
        digest.update(relative)
        digest.update(len(content).to_bytes(8, "big"))
        digest.update(content)
    return len(files), total, digest.hexdigest()


def git_value(root: Path, *args: str) -> str | None:
    if not (root / ".git").exists():
        return None
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        check=False,
        capture_output=True,
        text=True,
        timeout=5,
    )
    value = result.stdout.strip()
    return value if result.returncode == 0 and value else None


def inspect_skill(path: Path) -> SkillResult:
    issues: list[str] = []
    warnings: list[str] = []
    skill_file = path / "SKILL.md"
    kind = "skill"
    entrypoint_chars = 0
    declared_name: str | None = None
    description: str | None = None
    if not skill_file.is_file():
        if any(item.name == "SKILL.md" for item in visible_files(path)):
            kind = "source-container"
            warnings.append("源码容器：嵌套技能单独发现，不计入根级技能数")
        else:
            kind = "invalid-entry"
            issues.append("缺少 SKILL.md")
    else:
        try:
            text = skill_file.read_text(encoding="utf-8")
            entrypoint_chars = len(text)
            fields = parse_frontmatter(text)
            declared_name = fields.get("name")
            description = fields.get("description")
            if not declared_name:
                issues.append("缺少 frontmatter.name")
            elif declared_name != path.name:
                warnings.append(f"兼容别名需保留或核对：name={declared_name}")
            if declared_name and not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", declared_name):
                warnings.append("历史 name 不符合小写连字符规范；更名需核对调用入口")
            if not description:
                issues.append("缺少 frontmatter.description")
        except UnicodeDecodeError:
            issues.append("SKILL.md 不是 UTF-8")
    count, size, sha256 = digest_tree(path)
    return SkillResult(
        name=path.name,
        path=str(path),
        declared_name=declared_name,
        description=description,
        file_count=count,
        byte_count=size,
        sha256=sha256,
        git_origin=git_value(path, "remote", "get-url", "origin"),
        git_revision=git_value(path, "rev-parse", "HEAD"),
        issues=issues,
        kind=kind,
        warnings=warnings,
        entrypoint_chars=entrypoint_chars,
    )


def resolve_existing(path: Path) -> Path | None:
    try:
        return path.resolve(strict=True)
    except (FileNotFoundError, OSError, RuntimeError):
        return None


def inspect_tool(name: str, path: Path, central: Path) -> ToolResult:
    central_skills = {
        entry.name: entry
        for entry in central.iterdir()
        if not entry.name.startswith(".") and (entry / "SKILL.md").is_file()
    } if central.is_dir() else {}
    if not path.is_dir():
        return ToolResult(
            name, str(path), False, 0, 0, 0, 0,
            missing_shared_names=sorted(central_skills),
        )
    entries = [entry for entry in path.iterdir() if not entry.name.startswith(".")]
    symlinks = [entry for entry in entries if entry.is_symlink()]
    broken = [entry for entry in symlinks if resolve_existing(entry) is None]
    central_targets = {resolve_existing(entry) for entry in central_skills.values()}
    central_targets.discard(None)
    linked = sum(resolve_existing(entry) in central_targets for entry in symlinks)
    available_entries = {
        entry.name: entry for entry in entries if (entry / "SKILL.md").is_file()
    }
    local = [entry for entry in entries if entry.is_dir() and not entry.is_symlink()]
    same_name = [entry for entry in local if entry.name in central_skills]
    divergent = []
    for entry in same_name:
        local_file = entry / "SKILL.md"
        source_file = central_skills[entry.name] / "SKILL.md"
        if not local_file.is_file():
            continue
        local_bytes = local_file.read_bytes()
        source_bytes = source_file.read_bytes()
        if local_bytes == source_bytes:
            continue
        local_text = local_bytes.decode("utf-8", errors="replace")
        source_text = source_bytes.decode("utf-8", errors="replace")
        divergent.append({
            "name": entry.name,
            "local_path": str(local_file),
            "source_path": str(source_file),
            "local_sha256": hashlib.sha256(local_bytes).hexdigest(),
            "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
            "body_differs": FRONTMATTER.sub("", local_text, count=1).strip()
            != FRONTMATTER.sub("", source_text, count=1).strip(),
        })
    return ToolResult(
        name, str(path), True, len(entries), len(symlinks), len(broken), linked,
        len(available_entries), len(local), len(same_name),
        sorted(divergent, key=lambda item: str(item["name"])),
        sorted(set(central_skills) - set(available_entries)),
    )


def read_json_object(path: Path, issues: list[str]) -> dict:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(value, dict):
            raise ValueError("expected an object")
        return value
    except (OSError, UnicodeError, ValueError):
        issues.append(f"无法读取 JSON 对象：{path}")
        return {}


def inspect_claude_plugins(home: Path) -> dict[str, object]:
    """只输出插件状态及来源诊断，不复制设置、环境变量或凭据。"""
    root = home / ".claude"
    issues: list[str] = []
    settings = read_json_object(root / "settings.json", issues)
    enabled = settings.get("enabledPlugins", {})
    installed = read_json_object(root / "plugins/installed_plugins.json", issues)
    known = read_json_object(root / "plugins/known_marketplaces.json", issues)
    if not isinstance(enabled, dict):
        issues.append("enabledPlugins 不是对象")
        enabled = {}
    registry = installed.get("plugins", {})
    if not isinstance(registry, dict):
        issues.append("installed_plugins.plugins 不是对象")
        registry = {}
    marketplaces = []
    for name, item in sorted(known.items()):
        if not isinstance(item, dict):
            issues.append(f"市场 {name} 的记录不是对象")
            continue
        source = item.get("source", {})
        if not isinstance(source, dict):
            issues.append(f"市场 {name} 的 source 不是对象")
            source = {}
        kind = source.get("source")
        source_path = source.get("path") if kind == "directory" else None
        missing = kind == "directory" and (
            not isinstance(source_path, str) or not source_path or not Path(source_path).is_dir()
        )
        temporary = False
        if isinstance(source_path, str):
            resolved = Path(source_path).resolve(strict=False)
            temporary = any(resolved.is_relative_to(base) for base in (
                Path("/private/tmp"), Path("/tmp"),
                Path("/private/var/folders"), Path("/var/folders"),
            ))
        marketplaces.append({
            "name": name,
            "source_type": kind,
            "source_path": source_path,
            "source_repo": source.get("repo") if kind == "github" else None,
            "missing_local_source": missing,
            "temporary_local_source": temporary,
        })
    plugins = []
    for name, records in sorted(registry.items()):
        if not isinstance(records, list):
            issues.append(f"插件 {name} 的安装记录不是列表")
            continue
        for record in records:
            if not isinstance(record, dict):
                issues.append(f"插件 {name} 的安装条目不是对象")
                continue
            location = record.get("installPath")
            plugins.append({
                "name": name,
                "scope": record.get("scope"),
                "version": record.get("version"),
                "enabled_in_user_settings": enabled.get(name),
                "install_path": location,
                "cache_exists": isinstance(location, str) and Path(location).is_dir(),
            })
    return {
        "scope": "本机 Claude Code 用户设置与安装记录；不推断桌面账号、项目覆盖或真实调用",
        "installed_count": len(registry),
        "enabled_installed_count": sum(enabled.get(name) is True for name in registry),
        "marketplace_count": len(marketplaces),
        "missing_source_count": sum(item["missing_local_source"] for item in marketplaces),
        "temporary_source_count": sum(item["temporary_local_source"] for item in marketplaces),
        "plugins": plugins,
        "marketplaces": marketplaces,
        "issues": issues,
    }


def build_report(
    central: Path, home: Path, retired_skills: Iterable[str] = (),
) -> dict[str, object]:
    central = central.expanduser().absolute()
    skills = []
    if central.is_dir():
        skills = [
            inspect_skill(entry)
            for entry in sorted(central.iterdir(), key=lambda item: item.name.casefold())
            if entry.is_dir() and not entry.name.startswith(".")
        ]
    tools = [
        inspect_tool(name, home / relative, central)
        for name, relative in TOOL_PATHS.items()
    ]
    issue_count = sum(len(skill.issues) for skill in skills)
    skill_count = sum(skill.kind == "skill" for skill in skills)
    source_count = sum(skill.kind == "source-container" for skill in skills)
    warning_count = sum(len(skill.warnings) for skill in skills)
    broken_count = sum(tool.broken_symlink_count for tool in tools)
    plugins = inspect_claude_plugins(home)
    retired = sorted(set(retired_skills).intersection(skill.name for skill in skills))
    divergent_count = sum(len(tool.divergent_local_skills) for tool in tools)
    plugin_issue_count = (
        plugins["missing_source_count"] + plugins["temporary_source_count"]
        + len(plugins["issues"])
    )
    return {
        "schema_version": 3,
        "generated_at": datetime.now().astimezone().isoformat(timespec="seconds"),
        "read_only": True,
        "central": {
            "path": str(central),
            "exists": central.is_dir(),
            "skill_count": skill_count,
            "directory_count": len(skills),
            "source_container_count": source_count,
            "issue_count": issue_count,
        },
        "skills": [asdict(skill) for skill in skills],
        "tools": [asdict(tool) for tool in tools],
        "claude_plugins": plugins,
        "retired_skills_still_active": retired,
        "summary": {
            "skill_count": skill_count,
            "directory_count": len(skills),
            "source_container_count": source_count,
            "compatibility_warning_count": warning_count,
            "entrypoint_chars": sum(skill.entrypoint_chars for skill in skills),
            "skill_issue_count": issue_count,
            "broken_symlink_count": broken_count,
            "divergent_local_skill_count": divergent_count,
            "missing_plugin_source_count": plugins["missing_source_count"],
            "temporary_plugin_source_count": plugins["temporary_source_count"],
            "retired_skill_count": len(retired),
            "healthy": central.is_dir() and not any((
                issue_count, broken_count, plugin_issue_count, len(retired),
            )),
        },
    }


def markdown_report(report: dict[str, object]) -> str:
    central = report["central"]
    summary = report["summary"]
    assert isinstance(central, dict) and isinstance(summary, dict)
    lines = [
        "# AI 技能库只读审计报告",
        "",
        f"- 生成时间：{report['generated_at']}",
        f"- 中心技能源：`{central['path']}`",
        f"- 中心技能数：{summary['skill_count']}",
        f"- 根目录项：{summary['directory_count']}（其中源码容器 {summary['source_container_count']}）",
        f"- 技能规范问题：{summary['skill_issue_count']}",
        f"- 兼容提示：{summary['compatibility_warning_count']}（不自动更名或删除）",
        f"- 断链：{summary['broken_symlink_count']}",
        f"- 与共享源不同的本地入口：{summary['divergent_local_skill_count']}（差异不代表本地版错误，不自动覆盖）",
        f"- 失效插件来源：{summary['missing_plugin_source_count']}；临时目录来源：{summary['temporary_plugin_source_count']}",
        f"- 已声明弃用但仍在活动库：{summary['retired_skill_count']}",
        f"- 静态检查：{'通过' if summary['healthy'] else '有结构或入口问题，按具体条目核查'}（不代表业务调用验收；独立副本差异不计为故障）",
        "",
        "## AI 工具入口",
        "",
        "| 工具 | 路径 | 入口数 | 软链接 | 指向中心源 | 断链 |",
        "|---|---|---:|---:|---:|---:|",
    ]
    for tool in report["tools"]:
        assert isinstance(tool, dict)
        lines.append(
            f"| {tool['name']} | `{tool['path']}` | {tool['entry_count']} | "
            f"{tool['symlink_count']} | {tool['central_link_count']} | {tool['broken_symlink_count']} |"
        )
    lines.extend([
        "", "## 多端副本与入口差异", "",
        "以下只比较本机技能目录；缺少同名入口不等于能力不可用，也不构成删除依据。",
        "Codex 等宿主可能直接发现共享源；插件、项目与桌面账号还可能提供其他入口。",
        "Kimi Code CLI 直接读取共享源，“缺少共享同名入口”对它不是缺口；它自有目录中的同名入口反而会重复加载，需人工核对。", "",
        "| 工具 | 根级技能 | 本地目录 | 同名本地副本 | 入口内容不同 | 缺少共享同名入口 |",
        "|---|---:|---:|---:|---:|---:|",
    ])
    for tool in report["tools"]:
        if not tool["exists"]:
            continue
        lines.append(
            f"| {tool['name']} | {tool['root_skill_count']} | {tool['local_directory_count']} | "
            f"{tool['same_name_local_count']} | {len(tool['divergent_local_skills'])} | "
            f"{len(tool['missing_shared_names'])} |"
        )
    for tool in report["tools"]:
        for item in tool["divergent_local_skills"]:
            difference = "正文也不同" if item["body_differs"] else "正文相同，元信息不同"
            lines.append(f"\n- {tool['name']} / `{item['name']}`：{difference}；保留两版逐项审查。")
    plugins = report["claude_plugins"]
    lines.extend([
        "", "## Claude Code 插件来源", "",
        f"{plugins['scope']}。已装 {plugins['installed_count']} 个，用户设置启用 {plugins['enabled_installed_count']} 个。",
        "停用本身不是故障，也不能仅凭同类技能存在就断言全部能力已覆盖。", "",
    ])
    for market in plugins["marketplaces"]:
        if market["missing_local_source"] or market["temporary_local_source"]:
            status = "目录缺失" if market["missing_local_source"] else "当前存在但易失效"
            lines.append(f"- `{market['name']}`：{status}；来源 `{market['source_path']}`。")
    for issue in plugins["issues"]:
        lines.append(f"- {issue}")
    for name in report["retired_skills_still_active"]:
        lines.append(f"- 已声明弃用但未退出活动库：`{name}`；先确认备份与处置，不自动删除。")
    lines.extend(["", "## 需要处理的技能", ""])
    issues_found = False
    for skill in report["skills"]:
        assert isinstance(skill, dict)
        if skill["issues"]:
            issues_found = True
            lines.append(f"- `{skill['name']}`：{'\uff1b'.join(skill['issues'])}")
        if skill["warnings"]:
            issues_found = True
            lines.append(f"- `{skill['name']}`（提示）：{'；'.join(skill['warnings'])}")
    if not issues_found:
        lines.append("- 未发现技能文件夹规范问题。")
    lines.extend(["", "> 本报告由只读审计器生成，没有修改任何技能或软链接。", ""])
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="只读审计共享 AI 技能源及各工具入口")
    parser.add_argument("--central", type=Path, default=Path("~/.agents/skills"))
    parser.add_argument("--home", type=Path, default=Path.home())
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--strict", action="store_true", help="发现问题时返回非 0")
    parser.add_argument(
        "--retired-skill", action="append", default=[], metavar="NAME",
        help="检查已确认弃用的精确目录名，可重复；只报告，不移动或删除",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args.central, args.home.expanduser().absolute(), args.retired_skill)
    rendered = (
        json.dumps(report, ensure_ascii=False, indent=2) + "\n"
        if args.format == "json"
        else markdown_report(report)
    )
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 2 if args.strict and not report["summary"]["healthy"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
