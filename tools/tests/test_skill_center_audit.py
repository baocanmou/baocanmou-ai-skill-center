import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "skill_center_audit.py"
SPEC = importlib.util.spec_from_file_location("skill_center_audit", SCRIPT)
assert SPEC and SPEC.loader
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)


class SkillCenterAuditTests(unittest.TestCase):
    def write_skill(self, root, name, description="Example", body="Same body"):
        target = root / name
        target.mkdir(parents=True)
        (target / "SKILL.md").write_text(
            f"---\nname: {name}\ndescription: {description}\n---\n{body}\n",
            encoding="utf-8",
        )
        return target

    def write_json(self, path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_local_copy_drift_and_absent_names_do_not_include_source_containers(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            local = home / ".claude/skills"
            for name in ["same", "metadata", "body", "not-linked"]:
                self.write_skill(central, name)
            self.write_skill(central / "source/skills", "nested")
            self.write_skill(local, "same")
            self.write_skill(local, "metadata", description="Local translation")
            self.write_skill(local, "body", body="Local workflow")
            before = (local / "body/SKILL.md").read_bytes()
            result = AUDIT.inspect_tool("Claude Code", local, central)
            self.assertEqual(result.root_skill_count, 3)
            self.assertEqual(result.same_name_local_count, 3)
            self.assertEqual(result.missing_shared_names, ["not-linked"])
            self.assertEqual(len(result.divergent_local_skills), 2)
            diffs = {item["name"]: item for item in result.divergent_local_skills}
            self.assertTrue(diffs["body"]["body_differs"])
            self.assertFalse(diffs["metadata"]["body_differs"])
            self.assertEqual((local / "body/SKILL.md").read_bytes(), before)

    def test_nested_shared_symlink_counts_as_central_link(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            source = self.write_skill(central / "source/skills", "nested")
            (central / "alias").symlink_to(source, target_is_directory=True)
            local = home / ".claude/skills"
            local.mkdir(parents=True)
            (local / "alias").symlink_to(central / "alias", target_is_directory=True)
            result = AUDIT.inspect_tool("Claude Code", local, central)
            self.assertEqual(result.central_link_count, 1)
            self.assertEqual(result.missing_shared_names, [])

    def test_independent_valid_variant_is_not_a_broken_skill(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            self.write_skill(central, "demo")
            local = self.write_skill(home / ".claude/skills", "demo")
            with (local / "SKILL.md").open("a") as stream:
                stream.write("\nLocal instructions preserved.\n")
            before = (local / "SKILL.md").read_bytes()
            report = AUDIT.build_report(central, home)
            self.assertEqual(report["summary"]["divergent_local_skill_count"], 1)
            self.assertEqual(report["summary"]["broken_symlink_count"], 0)
            self.assertTrue(report["summary"]["healthy"])
            self.assertEqual((local / "SKILL.md").read_bytes(), before)

    def test_missing_and_temporary_plugin_sources_are_reported_without_secrets(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            central.mkdir(parents=True)
            settings = home / ".claude/settings.json"
            self.write_json(settings, {
                "enabledPlugins": {"kami@kami": False},
                "env": {"API_KEY": "must-not-appear-in-report"},
            })
            self.write_json(home / ".claude/plugins/known_marketplaces.json", {
                "kami": {"source": {"source": "directory", "path": "/tmp/missing-bcm-test-source"}},
            })
            before = settings.read_bytes()
            report = AUDIT.build_report(central, home)
            plugins = report["claude_plugins"]
            self.assertEqual(plugins["missing_source_count"], 1)
            self.assertEqual(plugins["temporary_source_count"], 1)
            self.assertFalse(report["summary"]["healthy"])
            self.assertNotIn("must-not-appear", json.dumps(report))
            self.assertEqual(settings.read_bytes(), before)

    def test_disabled_plugins_are_not_treated_as_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            central.mkdir(parents=True)
            cache = home / ".claude/plugins/cache/example/1"
            cache.mkdir(parents=True)
            self.write_json(home / ".claude/settings.json", {
                "enabledPlugins": {"example@example": False},
            })
            self.write_json(home / ".claude/plugins/installed_plugins.json", {
                "plugins": {"example@example": [{"installPath": str(cache), "version": "1"}]},
            })
            self.write_json(home / ".claude/plugins/known_marketplaces.json", {
                "example": {"source": {"source": "github", "repo": "owner/example"}},
            })
            report = AUDIT.build_report(central, home)
            self.assertEqual(report["claude_plugins"]["installed_count"], 1)
            self.assertEqual(report["claude_plugins"]["enabled_installed_count"], 0)
            self.assertTrue(report["summary"]["healthy"])

    def test_malformed_plugin_json_is_not_silently_healthy(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            central.mkdir(parents=True)
            settings = home / ".claude/settings.json"
            self.write_json(settings, ["not-an-object"])
            report = AUDIT.build_report(central, home)
            self.assertFalse(report["summary"]["healthy"])
            self.assertEqual(len(report["claude_plugins"]["issues"]), 1)

    def test_existing_temporary_plugin_source_is_still_flagged(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            source = home / "temporary-marketplace"
            source.mkdir()
            self.write_json(home / ".claude/plugins/known_marketplaces.json", {
                "example": {"source": {"source": "directory", "path": str(source)}},
            })
            plugins = AUDIT.inspect_claude_plugins(home)
            self.assertEqual(plugins["missing_source_count"], 0)
            self.assertEqual(plugins["temporary_source_count"], 1)

    def test_retirement_is_explicit_and_read_only(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            skill = self.write_skill(central, "retired")
            self.assertEqual(AUDIT.build_report(central, home)["retired_skills_still_active"], [])
            report = AUDIT.build_report(central, home, ["retired", "absent"])
            self.assertEqual(report["retired_skills_still_active"], ["retired"])
            self.assertFalse(report["summary"]["healthy"])
            self.assertTrue((skill / "SKILL.md").is_file())

    def test_folded_and_quoted_frontmatter(self):
        fields = AUDIT.parse_frontmatter('---\nname: demo\ndescription: >-\n  Explain useful\n  work.\nmetadata:\n  name: do-not-use\n---\n')
        self.assertEqual(fields['description'], 'Explain useful work.')
        self.assertEqual(fields['name'], 'demo')
        quoted = AUDIT.parse_frontmatter('---\nname: "demo"\ndescription: \'It\'\'s useful\'\n---\n')
        self.assertEqual(quoted['description'], "It's useful")

    def test_separates_source_containers_and_alias_warnings(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            central = root / 'skills'
            source = central / 'source/skills/actual'
            source.mkdir(parents=True)
            (source / 'SKILL.md').write_text('---\nname: actual\ndescription: Example\n---\n')
            alias = central / 'compatible'
            alias.symlink_to(source, target_is_directory=True)
            report = AUDIT.build_report(central, root)
            self.assertEqual(report['summary']['directory_count'], 2)
            self.assertEqual(report['summary']['skill_count'], 1)
            self.assertEqual(report['summary']['source_container_count'], 1)
            self.assertEqual(report['summary']['skill_issue_count'], 0)
            self.assertEqual(report['summary']['compatibility_warning_count'], 2)
            self.assertTrue(alias.is_symlink())

    def test_missing_root_is_not_created(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            missing = root / 'missing'
            report = AUDIT.build_report(missing, root)
            self.assertFalse(report['summary']['healthy'])
            self.assertFalse(missing.exists())

    def test_reports_valid_skill_and_tool_links(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            skill = central / "demo"
            skill.mkdir(parents=True)
            (skill / "SKILL.md").write_text(
                "---\nname: demo\ndescription: 演示技能\n---\n\n# 演示\n",
                encoding="utf-8",
            )
            target = home / ".codex/skills"
            target.mkdir(parents=True)
            (target / "demo").symlink_to(skill, target_is_directory=True)

            report = AUDIT.build_report(central, home)

            self.assertEqual(report["summary"]["skill_count"], 1)
            self.assertEqual(report["summary"]["skill_issue_count"], 0)
            codex = next(item for item in report["tools"] if item["name"] == "Codex")
            self.assertEqual(codex["central_link_count"], 1)
            self.assertEqual(codex["broken_symlink_count"], 0)

    def test_reports_invalid_skill_and_broken_link(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            (central / "invalid").mkdir(parents=True)
            target = home / ".claude/skills"
            target.mkdir(parents=True)
            (target / "missing").symlink_to(central / "missing", target_is_directory=True)

            report = AUDIT.build_report(central, home)

            self.assertEqual(report["summary"]["skill_issue_count"], 1)
            self.assertEqual(report["summary"]["broken_symlink_count"], 1)
            self.assertFalse(report["summary"]["healthy"])

    def test_markdown_is_chinese_and_marks_read_only(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            central = home / ".agents/skills"
            central.mkdir(parents=True)

            rendered = AUDIT.markdown_report(AUDIT.build_report(central, home))

            self.assertIn("AI 技能库只读审计报告", rendered)
            self.assertIn("没有修改任何技能或软链接", rendered)


if __name__ == "__main__":
    unittest.main()
