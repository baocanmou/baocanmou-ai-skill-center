use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

const SKILL_FILE: &str = "SKILL.md";
const COPY_MARKER: &str = ".baocanmou-managed-copy";
const MAX_SKILL_BYTES: u64 = 512 * 1024;
const MAX_PREVIEW_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PREVIEW_IMAGES: usize = 4;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CenterSnapshot {
    pub center_path: String,
    pub generated_at: u64,
    pub skills: Vec<SkillAsset>,
    pub tools: Vec<ToolStatus>,
    pub summary: SnapshotSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    pub asset_count: usize,
    pub ready_count: usize,
    pub attention_count: usize,
    pub connection_count: usize,
    pub chinese_ready_count: usize,
    pub screenshot_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillAsset {
    pub id: String,
    pub name_zh: String,
    pub name_en: String,
    pub summary_zh: String,
    pub summary_en: String,
    pub purpose_zh: String,
    pub purpose_en: String,
    pub features_zh: Vec<String>,
    pub features_en: Vec<String>,
    pub category: String,
    pub path: String,
    pub score: u8,
    pub status: String,
    pub risk_level: String,
    pub risk_flags: Vec<String>,
    pub file_count: usize,
    pub content_hash: String,
    pub modified_at: u64,
    pub translation_mode: String,
    pub preview_kind: String,
    pub preview_count: usize,
    pub connections: Vec<SkillConnection>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillConnection {
    pub tool_id: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub id: String,
    pub name: String,
    pub detected: bool,
    pub skills_path: String,
    pub reads_center: bool,
    pub linked_count: usize,
    pub conflict_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContent {
    pub skill_id: String,
    pub path: String,
    pub markdown: String,
    pub preview_images: Vec<SkillPreviewImage>,
    pub preview_kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPreviewImage {
    pub data_url: String,
    pub file_name: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationInput {
    pub skill_id: String,
    pub name_zh: String,
    pub summary_zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TranslationRecord {
    name_zh: String,
    summary_zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TranslationStore {
    translations: BTreeMap<String, TranslationRecord>,
}

#[derive(Debug, Clone)]
struct ToolSpec {
    id: &'static str,
    name: &'static str,
    /// Root directory relative to the home directory (`%USERPROFILE%` on Windows).
    root: &'static str,
    skills_dir: &'static str,
    /// Environment variable that relocates the root to an absolute path, when the tool supports one.
    root_env: Option<&'static str>,
    /// The tool scans the default shared center (`~/.agents/skills`) by itself, so a second
    /// managed link would load the same skill twice.
    reads_shared_center: bool,
}

fn tool_specs() -> [ToolSpec; 13] {
    [
        ToolSpec {
            id: "codex",
            name: "Codex",
            root: ".codex",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "claude",
            name: "Claude Code",
            root: ".claude",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "gemini",
            name: "Gemini CLI",
            root: ".gemini",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "cursor",
            name: "Cursor",
            root: ".cursor",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "hermes",
            name: "Hermes",
            root: ".hermes",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "zcode",
            name: "ZCode",
            root: ".zcode",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "opencode",
            name: "OpenCode",
            root: ".config/opencode",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "windsurf",
            name: "Windsurf",
            root: ".windsurf",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "kimi-code",
            name: "Kimi Code CLI",
            root: ".kimi-code",
            skills_dir: "skills",
            root_env: Some("KIMI_CODE_HOME"),
            reads_shared_center: true,
        },
        ToolSpec {
            id: "comate",
            name: "文心快码 Comate",
            root: ".comate",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "qwen-code",
            name: "Qwen Code",
            root: ".qwen",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "trae",
            name: "TRAE",
            root: ".trae",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
        ToolSpec {
            id: "trae-cn",
            name: "TRAE CN",
            root: ".trae-cn",
            skills_dir: "skills",
            root_env: None,
            reads_shared_center: false,
        },
    ]
}

fn tool_root(tool: &ToolSpec, home: &Path) -> PathBuf {
    resolve_tool_root(home, tool.root, tool.root_env.and_then(std::env::var_os))
}

fn resolve_tool_root(
    home: &Path,
    root: &str,
    override_root: Option<std::ffi::OsString>,
) -> PathBuf {
    override_root
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(root))
}

fn tool_skills_path(tool: &ToolSpec, home: &Path) -> PathBuf {
    tool_root(tool, home).join(tool.skills_dir)
}

/// True when the tool already discovers every center skill on its own. This only holds while
/// the center is the default `~/.agents/skills`; a custom `BAOCANMOU_SKILLS_HOME` falls back to
/// ordinary managed links.
fn reads_center_directly(tool: &ToolSpec, home: &Path, center: &Path) -> bool {
    tool.reads_shared_center
        && canonical_or_clean(center) == canonical_or_clean(&home.join(".agents").join("skills"))
}

pub fn scan() -> io::Result<CenterSnapshot> {
    let center = center_root()?;
    let home = home_root()?;
    let translations = load_translations().unwrap_or_default();
    let skills = scan_skill_assets(&center, &home, &translations)?;
    let tools = inspect_tools(&home, &center, &skills);
    let connection_count = skills
        .iter()
        .flat_map(|skill| &skill.connections)
        .filter(|connection| connection.mode == "link" || connection.mode == "copy")
        .count();
    let summary = SnapshotSummary {
        asset_count: skills.len(),
        ready_count: skills
            .iter()
            .filter(|skill| skill.status == "ready")
            .count(),
        attention_count: skills
            .iter()
            .filter(|skill| skill.status != "ready")
            .count(),
        connection_count,
        chinese_ready_count: skills
            .iter()
            .filter(|skill| {
                matches!(
                    skill.translation_mode.as_str(),
                    "native" | "curated" | "custom"
                )
            })
            .count(),
        screenshot_count: skills
            .iter()
            .filter(|skill| skill.preview_kind == "screenshot")
            .count(),
    };

    Ok(CenterSnapshot {
        center_path: display_path(&center),
        generated_at: now_seconds(),
        skills,
        tools,
        summary,
    })
}

fn scan_skill_assets(
    center: &Path,
    home: &Path,
    translations: &TranslationStore,
) -> io::Result<Vec<SkillAsset>> {
    let mut skills = Vec::new();
    // A read-only scan must fail for a missing/moved root, never create a new one.
    for entry in sorted_entries(center)? {
        let path = entry.path();
        if is_hidden(&entry.file_name()) || !path.join(SKILL_FILE).is_file() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if validate_id(&id).is_ok() {
            skills.push(inspect_skill(center, home, &id, &path, translations));
        }
    }
    skills.sort_by(|a, b| a.name_zh.cmp(&b.name_zh).then_with(|| a.id.cmp(&b.id)));
    Ok(skills)
}

pub fn read_skill(skill_id: &str) -> io::Result<SkillContent> {
    validate_id(skill_id)?;
    let center = center_root()?;
    let file = center.join(skill_id).join(SKILL_FILE);
    ensure_skill_path(&center, &file)?;
    let metadata = fs::metadata(&file)?;
    if metadata.len() > MAX_SKILL_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "SKILL.md exceeds the 512 KB safety limit",
        ));
    }
    let preview_images = read_preview_images(&center.join(skill_id))?;
    let preview_kind = if preview_images.is_empty() {
        "generated"
    } else {
        "screenshot"
    };
    Ok(SkillContent {
        skill_id: skill_id.to_owned(),
        path: display_path(&file),
        markdown: fs::read_to_string(&file)?,
        preview_images,
        preview_kind: preview_kind.to_owned(),
    })
}

pub fn save_translation(input: TranslationInput) -> io::Result<()> {
    validate_id(&input.skill_id)?;
    let name = input.name_zh.trim();
    let summary = input.summary_zh.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Chinese name must contain 1 to 80 characters",
        ));
    }
    if summary.is_empty() || summary.chars().count() > 400 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Chinese summary must contain 1 to 400 characters",
        ));
    }
    let center = center_root()?;
    ensure_skill_path(&center, &center.join(&input.skill_id).join(SKILL_FILE))?;

    let mut store = load_translations().unwrap_or_default();
    store.translations.insert(
        input.skill_id,
        TranslationRecord {
            name_zh: name.to_owned(),
            summary_zh: summary.to_owned(),
        },
    );
    write_translations(&store)
}

pub fn connect(skill_id: &str, tool_id: &str) -> io::Result<()> {
    validate_id(skill_id)?;
    let center = center_root()?;
    let home = home_root()?;
    let source = center.join(skill_id);
    ensure_skill_path(&center, &source.join(SKILL_FILE))?;
    let spec = find_tool(tool_id)?;
    if reads_center_directly(&spec, &home, &center) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} already reads {} directly; no link is needed",
                spec.name,
                display_path(&center)
            ),
        ));
    }
    let target_dir = tool_skills_path(&spec, &home);
    let target = target_dir.join(skill_id);
    fs::create_dir_all(&target_dir)?;

    match connection_mode(&source, &target) {
        ConnectionMode::Link | ConnectionMode::Copy => return Ok(()),
        ConnectionMode::Conflict | ConnectionMode::Broken => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "{} already exists and is not managed by BaoCanMou",
                    display_path(&target)
                ),
            ));
        }
        ConnectionMode::None => {}
    }

    create_managed_connection(&source, &target)
}

pub fn disconnect(skill_id: &str, tool_id: &str) -> io::Result<()> {
    validate_id(skill_id)?;
    let center = center_root()?;
    let home = home_root()?;
    let source = center.join(skill_id);
    let spec = find_tool(tool_id)?;
    let target = tool_skills_path(&spec, &home).join(skill_id);

    match connection_mode(&source, &target) {
        ConnectionMode::None => Ok(()),
        ConnectionMode::Link => fs::remove_file(target),
        ConnectionMode::Copy => fs::remove_dir_all(target),
        ConnectionMode::Broken | ConnectionMode::Conflict => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "refusing to remove an unmanaged or broken target",
        )),
    }
}

fn inspect_skill(
    center: &Path,
    home: &Path,
    id: &str,
    path: &Path,
    translations: &TranslationStore,
) -> SkillAsset {
    let skill_file = path.join(SKILL_FILE);
    let markdown = fs::read_to_string(&skill_file).unwrap_or_default();
    let metadata = parse_frontmatter(&markdown);
    let content_hash = sha256_text(&markdown);
    let curated = crate::curated::for_source(id, &content_hash);
    let name_en = metadata
        .get("name")
        .cloned()
        .unwrap_or_else(|| id.replace('-', " "));
    let summary_en = metadata
        .get("description_en")
        .or_else(|| {
            metadata
                .get("description")
                .filter(|value| !contains_cjk(value))
        })
        .cloned()
        .or_else(|| curated.map(|value| value.purpose_en.clone()))
        .or_else(|| metadata.get("description").cloned())
        .unwrap_or_default();
    let category = metadata
        .get("category")
        .filter(|value| {
            [
                "image",
                "design",
                "development",
                "content",
                "presentation",
                "video",
                "data",
                "security",
                "automation",
                "marketing",
                "research",
                "productivity",
                "general",
                "ai-assistant",
                "browser-automation",
                "data-visualization",
                "database",
                "devops",
            ]
            .contains(&value.as_str())
        })
        .cloned()
        .or_else(|| curated.map(|value| value.category.clone()))
        .unwrap_or_else(|| infer_category(&format!("{id} {name_en} {summary_en}")));
    let custom = translations.translations.get(id);
    let metadata_name_zh = metadata.get("name_zh").filter(|value| contains_cjk(value));
    let metadata_summary_zh = metadata
        .get("description_zh")
        .filter(|value| contains_cjk(value));
    let name_zh = custom
        .map(|value| value.name_zh.clone())
        .or_else(|| metadata_name_zh.cloned())
        .or_else(|| curated.map(|value| value.name_zh.clone()))
        .or_else(|| contains_cjk(&name_en).then(|| name_en.clone()))
        .unwrap_or_else(|| format!("中文名待核对 · {id}"));
    let purpose_zh = custom
        .map(|value| value.summary_zh.clone())
        .or_else(|| {
            metadata
                .get("purpose_zh")
                .filter(|value| contains_cjk(value))
                .cloned()
        })
        .or_else(|| metadata_summary_zh.cloned())
        .or_else(|| curated.map(|value| value.purpose_zh.clone()))
        .or_else(|| {
            metadata
                .get("description")
                .filter(|value| contains_cjk(value))
                .cloned()
        })
        .unwrap_or_else(|| "中文用途待核对，请查看原始说明；不根据名称猜测能力。".to_owned());
    let summary_zh = custom
        .map(|value| value.summary_zh.clone())
        .or_else(|| metadata_summary_zh.cloned())
        .unwrap_or_else(|| purpose_zh.clone());
    let purpose_en = metadata
        .get("purpose_en")
        .or_else(|| metadata.get("description_en"))
        .filter(|value| !contains_cjk(value))
        .cloned()
        .or_else(|| curated.map(|value| value.purpose_en.clone()))
        .unwrap_or_else(|| summary_en.clone());
    let native_purpose = ["purpose_zh", "description_zh", "description"]
        .iter()
        .any(|key| metadata.get(*key).is_some_and(|value| contains_cjk(value)));
    let translation_mode = if custom.is_some() {
        "custom"
    } else if (metadata_name_zh.is_some() || contains_cjk(&name_en)) && native_purpose {
        "native"
    } else if curated.is_some() {
        "curated"
    } else {
        "pending"
    };
    let (risk_level, risk_flags) = risk_assessment(&markdown);
    let file_count = count_files(path, 5).unwrap_or(0);
    let (fallback_zh, fallback_en) = feature_labels(path, &markdown, &risk_flags);
    let features_zh = metadata_features(&metadata, "features_zh")
        .or_else(|| curated.map(|value| value.features_zh.clone()))
        .unwrap_or(fallback_zh);
    let features_en = metadata_features(&metadata, "features_en")
        .or_else(|| curated.map(|value| value.features_en.clone()))
        .unwrap_or(fallback_en);
    let preview_count = find_preview_images(path).len();
    let preview_kind = if preview_count > 0 {
        "screenshot"
    } else {
        "generated"
    };
    let modified_at = fs::metadata(&skill_file)
        .and_then(|value| value.modified())
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or_default();
    let connections = tool_specs()
        .iter()
        .map(|tool| {
            let mode = if reads_center_directly(tool, home, center) {
                "native"
            } else {
                let target = tool_skills_path(tool, home).join(id);
                connection_mode(path, &target).as_str()
            };
            SkillConnection {
                tool_id: tool.id.to_owned(),
                mode: mode.to_owned(),
            }
        })
        .collect::<Vec<_>>();
    let has_frontmatter = metadata.contains_key("name") && metadata.contains_key("description");
    let score = readiness_score(
        !markdown.is_empty(),
        has_frontmatter,
        !summary_en.is_empty() || contains_cjk(&summary_zh),
        &risk_level,
        !content_hash.is_empty(),
        path.starts_with(center),
    );
    let status = if markdown.is_empty() {
        "invalid"
    } else if risk_level == "high" || !has_frontmatter {
        "attention"
    } else {
        "ready"
    };

    SkillAsset {
        id: id.to_owned(),
        name_zh,
        name_en,
        summary_zh,
        summary_en,
        purpose_zh,
        purpose_en,
        features_zh,
        features_en,
        category,
        path: display_path(path),
        score,
        status: status.to_owned(),
        risk_level,
        risk_flags,
        file_count,
        content_hash,
        modified_at,
        translation_mode: translation_mode.to_owned(),
        preview_kind: preview_kind.to_owned(),
        preview_count,
        connections,
    }
}

fn inspect_tools(home: &Path, center: &Path, skills: &[SkillAsset]) -> Vec<ToolStatus> {
    tool_specs()
        .iter()
        .map(|tool| {
            let root = tool_root(tool, home);
            let skills_path = root.join(tool.skills_dir);
            let reads_center = reads_center_directly(tool, home, center);
            let mut linked_count = 0;
            let mut conflict_count = 0;
            for skill in skills {
                let source = center.join(&skill.id);
                let target = skills_path.join(&skill.id);
                match (reads_center, connection_mode(&source, &target)) {
                    // Direct readers already see every center skill; a same-name entry in their
                    // own directory is a duplicate to review, never something to create.
                    (true, ConnectionMode::None) => linked_count += 1,
                    (true, _) => {
                        linked_count += 1;
                        conflict_count += 1;
                    }
                    (false, ConnectionMode::Link | ConnectionMode::Copy) => linked_count += 1,
                    (false, ConnectionMode::Broken | ConnectionMode::Conflict) => {
                        conflict_count += 1
                    }
                    (false, ConnectionMode::None) => {}
                }
            }
            ToolStatus {
                id: tool.id.to_owned(),
                name: tool.name.to_owned(),
                detected: root.exists(),
                skills_path: display_path(if reads_center { center } else { &skills_path }),
                reads_center,
                linked_count,
                conflict_count,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionMode {
    None,
    Link,
    Copy,
    Broken,
    Conflict,
}

impl ConnectionMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Link => "link",
            Self::Copy => "copy",
            Self::Broken => "broken",
            Self::Conflict => "conflict",
        }
    }
}

fn connection_mode(source: &Path, target: &Path) -> ConnectionMode {
    let Ok(metadata) = fs::symlink_metadata(target) else {
        return ConnectionMode::None;
    };
    if metadata.file_type().is_symlink() {
        let Ok(link) = fs::read_link(target) else {
            return ConnectionMode::Broken;
        };
        let resolved = if link.is_absolute() {
            link
        } else {
            target.parent().unwrap_or_else(|| Path::new(".")).join(link)
        };
        return if canonical_or_clean(&resolved) == canonical_or_clean(source) {
            ConnectionMode::Link
        } else if resolved.exists() {
            ConnectionMode::Conflict
        } else {
            ConnectionMode::Broken
        };
    }
    if metadata.is_dir() {
        let marker = target.join(COPY_MARKER);
        if let Ok(recorded_source) = fs::read_to_string(marker) {
            if canonical_or_clean(Path::new(recorded_source.trim())) == canonical_or_clean(source) {
                return ConnectionMode::Copy;
            }
        }
    }
    ConnectionMode::Conflict
}

#[cfg(unix)]
fn create_managed_connection(source: &Path, target: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn create_managed_connection(source: &Path, target: &Path) -> io::Result<()> {
    if std::os::windows::fs::symlink_dir(source, target).is_ok() {
        return Ok(());
    }
    copy_tree(source, target)?;
    fs::write(target.join(COPY_MARKER), display_path(source))
}

#[cfg(windows)]
fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn risk_assessment(markdown: &str) -> (String, Vec<String>) {
    let lower = markdown.to_lowercase();
    let checks = [
        (
            "destructive-command",
            ["rm -rf", "remove-item -recurse -force"].as_slice(),
        ),
        ("privilege-escalation", ["sudo ", "runas "].as_slice()),
        (
            "credential-access",
            ["cookie", "private key", "access token", "api key"].as_slice(),
        ),
        (
            "remote-execution",
            ["curl ", "wget ", "invoke-webrequest", "eval("].as_slice(),
        ),
    ];
    let mut flags = Vec::new();
    for (flag, patterns) in checks {
        if patterns.iter().any(|pattern| lower.contains(pattern)) {
            flags.push(flag.to_owned());
        }
    }
    let level = if flags
        .iter()
        .any(|flag| flag == "destructive-command" || flag == "privilege-escalation")
    {
        "high"
    } else if flags.is_empty() {
        "low"
    } else {
        "medium"
    };
    (level.to_owned(), flags)
}

fn readiness_score(
    has_skill_file: bool,
    has_frontmatter: bool,
    has_description: bool,
    risk_level: &str,
    has_hash: bool,
    in_center: bool,
) -> u8 {
    let structure = u8::from(has_skill_file) * 15 + u8::from(has_frontmatter) * 10;
    let understanding = u8::from(has_description) * 20;
    let portability = u8::from(in_center) * 20;
    let safety = match risk_level {
        "low" => 20,
        "medium" => 12,
        _ => 4,
    };
    let verifiability = u8::from(has_hash) * 15;
    structure + understanding + portability + safety + verifiability
}

fn parse_frontmatter(markdown: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let mut lines = markdown.lines();
    if markdown.len() as u64 > MAX_SKILL_BYTES || lines.next().map(str::trim) != Some("---") {
        return values;
    }
    let mut header = String::new();
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        header.push_str(line);
        header.push('\n');
    }
    if !closed {
        return values;
    }
    let Ok(parsed) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&header) else {
        return values;
    };
    // Only display fields at the root or directly under metadata are accepted.
    // Provider-specific nested dictionaries cannot overwrite the Skill contract.
    for key in [
        "name",
        "description",
        "name_zh",
        "description_zh",
        "description_en",
        "purpose_zh",
        "purpose_en",
        "features_zh",
        "features_en",
        "category",
    ] {
        let field = if key == "name" || key == "description" {
            parsed.get(key)
        } else {
            parsed
                .get("metadata")
                .and_then(|meta| meta.get(key))
                .or_else(|| parsed.get(key))
        };
        let value = field.and_then(|value| {
            if key.starts_with("features_") {
                if let Some(items) = value.as_sequence() {
                    return Some(
                        items
                            .iter()
                            .filter_map(|item| item.as_str())
                            .collect::<Vec<_>>()
                            .join("；"),
                    );
                }
            }
            value.as_str().map(str::to_owned)
        });
        if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
            values.insert(key.to_owned(), value.trim().to_owned());
        }
    }
    values
}

fn metadata_features(metadata: &HashMap<String, String>, key: &str) -> Option<Vec<String>> {
    let mut seen = HashSet::new();
    let labels = metadata
        .get(key)?
        .split([';', '；', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert((*value).to_owned()))
        .take(4)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    (!labels.is_empty()).then_some(labels)
}

fn matches_keyword(text: &str, keyword: &str) -> bool {
    if keyword.len() <= 3 && keyword.is_ascii() {
        text.split(|character: char| !character.is_ascii_alphanumeric())
            .any(|word| word == keyword)
    } else {
        text.contains(keyword)
    }
}

fn infer_category(text: &str) -> String {
    let lower = text.to_lowercase();
    let identifier = lower.split_whitespace().next().unwrap_or("");
    if identifier.contains("image-to-code") || lower.contains("image to code") {
        return "development".to_owned();
    }
    if ["ppt", "slide", "presentation", "courseware"]
        .iter()
        .any(|term| matches_keyword(identifier, term))
    {
        return "presentation".to_owned();
    }
    if ["video", "remotion", "caption", "pixel2motion"]
        .iter()
        .any(|term| matches_keyword(identifier, term))
    {
        return "video".to_owned();
    }
    if is_image_generation_capability(identifier) {
        return "image".to_owned();
    }
    if ["ppt", "slide", "presentation", "courseware"]
        .iter()
        .any(|term| matches_keyword(&lower, term))
    {
        return "presentation".to_owned();
    }
    if ["video", "remotion", "caption", "pixel2motion"]
        .iter()
        .any(|term| matches_keyword(&lower, term))
    {
        return "video".to_owned();
    }
    if is_image_generation_capability(&lower) {
        return "image".to_owned();
    }
    let categories = [
        (
            "design",
            ["design", "ui", "ux", "image", "visual", "logo"].as_slice(),
        ),
        (
            "development",
            ["code", "develop", "debug", "api", "frontend", "backend"].as_slice(),
        ),
        (
            "content",
            ["content", "copy", "article", "writing", "seo", "marketing"].as_slice(),
        ),
        ("presentation", ["ppt", "slide", "presentation"].as_slice()),
        (
            "video",
            ["video", "motion", "caption", "remotion"].as_slice(),
        ),
        (
            "data",
            ["data", "chart", "spreadsheet", "analytics", "database"].as_slice(),
        ),
        (
            "security",
            ["security", "threat", "vulnerability", "audit"].as_slice(),
        ),
        (
            "automation",
            ["automation", "browser", "workflow", "deploy", "ci"].as_slice(),
        ),
    ];
    for (category, keywords) in categories {
        if keywords
            .iter()
            .any(|keyword| matches_keyword(&lower, keyword))
        {
            return category.to_owned();
        }
    }
    "general".to_owned()
}

fn is_image_generation_capability(text: &str) -> bool {
    let lower = text.to_lowercase();
    let excluded = [
        "image-to-code",
        "image to code",
        "pixel2motion",
        "batch edit photo",
        "retouch portrait",
        "screenshot",
    ];
    if excluded.iter().any(|term| lower.contains(term)) {
        return false;
    }
    [
        "imagegen",
        "image generation",
        "image generator",
        "generate images",
        "generated image",
        "illustration",
        "comic",
        "logo-generator",
        "logo generator",
        "article visuals",
        "transparent image",
        "图像生成",
        "图片生成",
        "生成图片",
        "生成图像",
        "插画",
        "漫画",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

fn feature_labels(
    path: &Path,
    markdown: &str,
    risk_flags: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut zh = vec![
        "保留标准 SKILL.md 调用契约".to_owned(),
        "可在多个 AI 工具间复用".to_owned(),
    ];
    let mut en = vec![
        "Keeps the standard SKILL.md contract".to_owned(),
        "Reusable across multiple AI tools".to_owned(),
    ];
    let mut has_script = false;
    let mut has_reference = false;
    let mut has_visual = false;
    inspect_feature_files(
        path,
        4,
        &mut has_script,
        &mut has_reference,
        &mut has_visual,
    );
    if has_script {
        zh.push("包含可执行脚本，使用前应检查依赖和写入范围".to_owned());
        en.push("Includes executable scripts; review dependencies and write scope".to_owned());
    }
    if has_reference {
        zh.push("附带参考资料或模板，可减少重复整理".to_owned());
        en.push("Includes references or templates to reduce repeated preparation".to_owned());
    }
    if has_visual {
        zh.push("包含可展示的视觉示例或截图".to_owned());
        en.push("Includes visual examples or screenshots".to_owned());
    }
    let lower = markdown.to_lowercase();
    if lower.contains("browser") || lower.contains("http") {
        zh.push("可能访问浏览器或网络，执行时需确认授权".to_owned());
        en.push("May access a browser or network; confirm authorization before use".to_owned());
    }
    if !risk_flags.is_empty() {
        zh.push("检测到静态风险特征，需要阅读原文复核".to_owned());
        en.push("Static risk signals detected; read the source before use".to_owned());
    }
    zh.truncate(5);
    en.truncate(5);
    (zh, en)
}

fn inspect_feature_files(
    path: &Path,
    depth: usize,
    script: &mut bool,
    reference: &mut bool,
    visual: &mut bool,
) {
    if depth == 0 || !path.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if entry_path.is_dir() {
            if matches!(
                name.as_str(),
                "references" | "reference" | "templates" | "examples" | "docs"
            ) {
                *reference = true;
            }
            inspect_feature_files(&entry_path, depth - 1, script, reference, visual);
            continue;
        }
        let extension = entry_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        if matches!(
            extension.as_str(),
            "py" | "sh" | "bash" | "js" | "mjs" | "ts" | "rb" | "ps1"
        ) {
            *script = true;
        }
        if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif") {
            *visual = true;
        }
    }
}

fn find_preview_images(path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    collect_preview_images(path, 5, &mut candidates);
    candidates.sort_by_key(|candidate| {
        let full_path = candidate.to_string_lossy().to_lowercase();
        let name = candidate
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        let duplicate_priority = if name.contains("thumb") || full_path.contains("/thumbnails/") {
            1
        } else {
            0
        };
        let purpose_priority = if full_path.contains("/examples/")
            || full_path.contains("/example/")
            || full_path.contains("/showcase/")
            || full_path.contains("/samples/")
            || full_path.contains("/gallery/")
            || name.contains("screenshot")
            || name.contains("preview")
        {
            0
        } else if name.contains("cover") || name.contains("example") || name.contains("sample") {
            1
        } else {
            2
        };
        (duplicate_priority, purpose_priority, name, full_path)
    });
    candidates.retain(|candidate| {
        fs::metadata(candidate)
            .map(|value| value.len() > 0 && value.len() <= MAX_PREVIEW_BYTES)
            .unwrap_or(false)
    });
    candidates
}

fn collect_preview_images(path: &Path, depth: usize, output: &mut Vec<PathBuf>) {
    if depth == 0 || !path.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if entry_path.is_dir() {
            let directory = entry.file_name().to_string_lossy().to_lowercase();
            if matches!(
                directory.as_str(),
                "node_modules" | "target" | "dist" | "build" | "vendor" | "__pycache__" | "icons"
            ) {
                continue;
            }
            collect_preview_images(&entry_path, depth - 1, output);
            continue;
        }
        let extension = entry_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif") {
            output.push(entry_path);
        }
    }
}

fn read_preview_images(path: &Path) -> io::Result<Vec<SkillPreviewImage>> {
    let mut previews = Vec::new();
    for image in select_preview_images(path, MAX_PREVIEW_IMAGES) {
        let extension = image
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mime = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            _ => continue,
        };
        let file_name = image
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("preview")
            .to_owned();
        let label = preview_label(path, &image);
        let encoded = base64::engine::general_purpose::STANDARD.encode(fs::read(&image)?);
        previews.push(SkillPreviewImage {
            data_url: format!("data:{mime};base64,{encoded}"),
            file_name,
            label,
        });
    }
    Ok(previews)
}

fn select_preview_images(path: &Path, limit: usize) -> Vec<PathBuf> {
    let candidates = find_preview_images(path);
    let mut selected = Vec::new();
    let mut selected_paths = HashSet::new();
    let mut style_directories = HashSet::new();

    for candidate in &candidates {
        let lower = candidate.to_string_lossy().to_lowercase();
        let is_example = [
            "/examples/",
            "/example/",
            "/showcase/",
            "/samples/",
            "/gallery/",
        ]
        .iter()
        .any(|segment| lower.contains(segment));
        let parent = candidate
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        if is_example && style_directories.insert(parent) {
            selected_paths.insert(candidate.clone());
            selected.push(candidate.clone());
            if selected.len() == limit {
                return selected;
            }
        }
    }

    for candidate in candidates {
        if selected_paths.insert(candidate.clone()) {
            selected.push(candidate);
            if selected.len() == limit {
                break;
            }
        }
    }
    selected
}

fn preview_label(root: &Path, image: &Path) -> String {
    let file_name = image
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("preview");
    let parent_name = image
        .parent()
        .filter(|parent| *parent != root)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str());
    match parent_name {
        Some(parent) if !parent.is_empty() => format!("{parent} / {file_name}"),
        _ => file_name.to_owned(),
    }
}

fn contains_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character as u32, 0x3400..=0x9fff | 0xf900..=0xfaff))
}

fn count_files(path: &Path, depth: usize) -> io::Result<usize> {
    if depth == 0 || !path.is_dir() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_file() {
            count += 1;
        } else if file_type.is_dir() {
            count += count_files(&entry.path(), depth - 1)?;
        }
    }
    Ok(count)
}

fn sorted_entries(path: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

fn validate_id(value: &str) -> io::Result<()> {
    let path = Path::new(value);
    let valid = !value.is_empty()
        && value != "."
        && value != ".."
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && path.components().count() == 1;
    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid skill identifier",
        ))
    }
}

fn ensure_skill_path(center: &Path, candidate: &Path) -> io::Result<()> {
    let center = fs::canonicalize(center)?;
    let candidate = fs::canonicalize(candidate)?;
    if candidate.starts_with(center) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "skill path leaves the center root",
        ))
    }
}

fn find_tool(tool_id: &str) -> io::Result<ToolSpec> {
    tool_specs()
        .into_iter()
        .find(|tool| tool.id == tool_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unknown tool identifier"))
}

fn center_root() -> io::Result<PathBuf> {
    if let Some(custom) = std::env::var_os("BAOCANMOU_SKILLS_HOME") {
        return Ok(PathBuf::from(custom));
    }
    Ok(home_root()?.join(".agents").join("skills"))
}

fn state_file() -> io::Result<PathBuf> {
    Ok(home_root()?
        .join(".baocanmou")
        .join("skill-center")
        .join("translations.json"))
}

fn home_root() -> io::Result<PathBuf> {
    dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory is unavailable"))
}

fn load_translations() -> io::Result<TranslationStore> {
    let path = state_file()?;
    if !path.exists() {
        return Ok(TranslationStore::default());
    }
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_translations(store: &TranslationStore) -> io::Result<()> {
    let path = state_file()?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid state path"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join("translations.json.tmp");
    let content = serde_json::to_vec_pretty(store).map_err(io::Error::other)?;
    fs::write(&temporary, content)?;
    fs::rename(temporary, path)
}

fn sha256_text(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn canonical_or_clean(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_parser_reads_supported_fields() {
        let parsed = parse_frontmatter("---\nname: code-review\ndescription: Review code safely\nname_zh: 代码评审\n---\n# Body");
        assert_eq!(parsed.get("name"), Some(&"code-review".to_owned()));
        assert_eq!(parsed.get("name_zh"), Some(&"代码评审".to_owned()));
    }

    #[test]
    fn frontmatter_reads_folded_text_and_direct_metadata_only() {
        let parsed = parse_frontmatter("---\nname: demo\ndescription: >-\n  Make useful\n  images.\nmetadata:\n  name_zh: 中文名\n  features_zh: [横版图片, 中文标注]\n  description_en: \"Read: \\\"quoted\\\" text\"\n  openclaw:\n    name: do-not-override\n    name_zh: 不得覆盖\n---\n# Body\nname: also-ignore");
        assert_eq!(parsed.get("description").unwrap(), "Make useful images.");
        assert_eq!(parsed.get("name").unwrap(), "demo");
        assert_eq!(parsed.get("name_zh").unwrap(), "中文名");
        assert_eq!(parsed.get("features_zh").unwrap(), "横版图片；中文标注");
        assert_eq!(
            parsed.get("description_en").unwrap(),
            "Read: \"quoted\" text"
        );
        assert!(parse_frontmatter("---\nname: [broken\n---").is_empty());
        assert!(parse_frontmatter("---\nname: demo").is_empty());
    }

    #[test]
    fn asset_uses_curated_purpose_features_and_category() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("example-ppt");
        fs::create_dir(&path).unwrap();
        fs::write(path.join(SKILL_FILE), "---\nname: example-ppt\ndescription: 生成横版图片\nmetadata:\n  name_zh: 中文信息图\n  purpose_zh: 生成图片，不是可编辑PPT。\n  description_en: Generate images, not editable slides.\n  category: image\n  features_zh: 横版图片；中文标注；横版图片\n  features_en: Wide images;Chinese labels\n---\n# Example").unwrap();
        let mut translations = TranslationStore::default();
        let skill = inspect_skill(
            root.path(),
            root.path(),
            "example-ppt",
            &path,
            &translations,
        );
        assert_eq!(skill.category, "image");
        assert_eq!(skill.name_zh, "中文信息图");
        assert_eq!(skill.purpose_zh, "生成图片，不是可编辑PPT。");
        assert_eq!(skill.features_zh, vec!["横版图片", "中文标注"]);
        assert_eq!(skill.features_en, vec!["Wide images", "Chinese labels"]);
        assert_eq!(skill.summary_en, "Generate images, not editable slides.");
        assert_eq!(skill.purpose_en, skill.summary_en);

        translations.translations.insert(
            "example-ppt".into(),
            TranslationRecord {
                name_zh: "用户命名".into(),
                summary_zh: "用户自定义用途".into(),
            },
        );
        let custom = inspect_skill(
            root.path(),
            root.path(),
            "example-ppt",
            &path,
            &translations,
        );
        assert_eq!(custom.name_zh, "用户命名");
        assert_eq!(custom.summary_zh, "用户自定义用途");
        assert_eq!(custom.purpose_zh, "用户自定义用途");
    }

    #[test]
    fn scan_ignores_source_containers_and_does_not_create_missing_roots() {
        let root = tempfile::tempdir().unwrap();
        let center = root.path().join("skills");
        let translations = TranslationStore::default();
        assert!(scan_skill_assets(&center, root.path(), &translations).is_err());
        assert!(!center.exists());
        fs::create_dir_all(center.join("source-container")).unwrap();
        fs::create_dir_all(center.join("demo")).unwrap();
        fs::write(
            center.join("demo/SKILL.md"),
            "---\nname: demo\ndescription: Test\n---",
        )
        .unwrap();
        let skills = scan_skill_assets(&center, root.path(), &translations).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "demo");
    }

    #[test]
    #[ignore = "read-only integration check; requires an explicitly selected local skills root"]
    fn live_catalog_metadata_roundtrip() {
        let center =
            PathBuf::from(std::env::var_os("BAOCANMOU_SKILLS_HOME").expect("explicit root"));
        let expected: usize = std::env::var("BAOCANMOU_EXPECTED_SKILL_COUNT")
            .expect("expected count")
            .parse()
            .unwrap();
        let home = tempfile::tempdir().unwrap();
        let skills = scan_skill_assets(&center, home.path(), &TranslationStore::default()).unwrap();
        assert_eq!(skills.len(), expected);
        for skill in &skills {
            assert!(
                matches!(skill.translation_mode.as_str(), "native" | "curated"),
                "{}",
                skill.id
            );
            assert!(contains_cjk(&skill.name_zh), "{}", skill.id);
            assert!(contains_cjk(&skill.purpose_zh), "{}", skill.id);
            assert!(!contains_cjk(&skill.purpose_en), "{}", skill.id);
            assert!(skill.features_zh.len() >= 3, "{}", skill.id);
            assert!(!skill.summary_en.is_empty(), "{}", skill.id);
        }
        for id in [
            "last30days",
            "guizang-ppt-skill",
            "autumn-campus-defense-ppt",
            "image-to-code-skill",
            "baocut",
            "qianwen-payment",
            "qianwen-vision",
            "typesafe-ai",
        ] {
            // Representative output is optional: a user may have retired a Skill.
            // The assertions above still cover every currently installed asset.
            let Some(skill) = skills.iter().find(|skill| skill.id == id) else {
                continue;
            };
            println!(
                "{} | {} | {} | {}",
                skill.id,
                skill.category,
                skill.purpose_zh,
                skill.features_zh.join("；")
            );
        }
        println!(
            "Verified {} actual Skill cards through the desktop parser",
            skills.len()
        );
        if let Some(output) = std::env::var_os("BAOCANMOU_TEST_CATALOG_OUTPUT") {
            fs::write(output, serde_json::to_vec_pretty(&skills).unwrap()).unwrap();
        }
    }

    #[test]
    fn separates_image_generation_from_image_to_code() {
        assert_eq!(
            infer_category("comic-explainer-illustration generates images"),
            "image"
        );
        assert_eq!(
            infer_category("logo-generator creates a brand mark"),
            "image"
        );
        assert_eq!(
            infer_category("image-to-code frontend conversion"),
            "development"
        );
        assert_eq!(
            infer_category("illustrated business ppt presentation"),
            "presentation"
        );
        assert_eq!(infer_category("animated image video"), "video");
    }

    #[test]
    fn short_keywords_must_be_complete_words() {
        assert_eq!(
            infer_category("payment balance and recharge page guidance"),
            "general"
        );
        assert_eq!(infer_category("ui layout"), "design");
        assert!(!matches_keyword("building guidance", "ui"));
        assert!(!matches_keyword("specialization", "ci"));
        assert!(matches_keyword("ui-ux workflow", "ui"));
    }

    #[test]
    fn unknown_or_changed_sources_are_pending_not_guessed() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("skill");
        fs::create_dir(&path).unwrap();
        let source =
            "---\nname: qianwen-payment\ndescription: Changed balance and recharge guidance\n---\n";
        fs::write(path.join(SKILL_FILE), source).unwrap();
        for id in ["qianwen-payment", "unreviewed-skill"] {
            let skill = inspect_skill(
                root.path(),
                root.path(),
                id,
                &path,
                &TranslationStore::default(),
            );
            assert_eq!(skill.translation_mode, "pending");
            assert!(skill.name_zh.contains("待核对"));
            assert!(skill.purpose_zh.contains("待核对"));
            assert!(!skill.purpose_zh.contains("界面"));
            assert_eq!(skill.purpose_en, "Changed balance and recharge guidance");
        }
        assert_eq!(fs::read_to_string(path.join(SKILL_FILE)).unwrap(), source);
    }

    #[test]
    fn reads_up_to_four_real_preview_images_and_skips_icons() {
        let root = std::env::temp_dir().join(format!(
            "baocanmou-preview-test-{}-{}",
            std::process::id(),
            now_seconds()
        ));
        let examples = root.join("examples");
        let icons = root.join("icons");
        let quirky = examples.join("quirky-sketch");
        let warm = examples.join("warm-storybook");
        let product = examples.join("product-proposal");
        fs::create_dir_all(&quirky).expect("create quirky examples");
        fs::create_dir_all(&warm).expect("create warm examples");
        fs::create_dir_all(&product).expect("create product examples");
        fs::create_dir_all(&icons).expect("create icons");
        fs::write(quirky.join("style-a.png"), [1_u8, 2, 3]).expect("write preview");
        fs::write(warm.join("style-b.jpg"), [1_u8, 2, 3]).expect("write preview");
        fs::write(product.join("style-c.webp"), [1_u8, 2, 3]).expect("write preview");
        fs::write(quirky.join("style-d.gif"), [1_u8, 2, 3]).expect("write preview");
        fs::write(quirky.join("style-e.png"), [1_u8, 2, 3]).expect("write preview");
        fs::write(icons.join("app-icon.png"), [1_u8]).expect("write icon");

        let candidates = find_preview_images(&root);
        let previews = read_preview_images(&root).expect("read previews");
        assert_eq!(candidates.len(), 5);
        assert_eq!(previews.len(), 4);
        assert!(previews
            .iter()
            .all(|preview| preview.data_url.starts_with("data:image/")));
        assert!(previews
            .iter()
            .all(|preview| preview.file_name.starts_with("style-")));
        let first_three_styles = previews
            .iter()
            .take(3)
            .map(|preview| preview.label.split(" / ").next().unwrap_or(""))
            .collect::<HashSet<_>>();
        assert_eq!(first_three_styles.len(), 3);

        fs::remove_dir_all(root).expect("remove preview fixture");
    }

    #[test]
    fn host_table_has_unique_ids_and_domestic_paths() {
        let specs = tool_specs();
        let ids = specs.iter().map(|tool| tool.id).collect::<HashSet<_>>();
        assert_eq!(ids.len(), specs.len());
        let home = Path::new("/home/demo");
        let paths = specs
            .iter()
            .map(|tool| {
                (
                    tool.id,
                    resolve_tool_root(home, tool.root, None).join(tool.skills_dir),
                )
            })
            .collect::<HashMap<_, _>>();
        assert_eq!(paths["kimi-code"], home.join(".kimi-code/skills"));
        assert_eq!(paths["comate"], home.join(".comate/skills"));
        assert_eq!(paths["qwen-code"], home.join(".qwen/skills"));
        assert_eq!(paths["trae"], home.join(".trae/skills"));
        assert_eq!(paths["trae-cn"], home.join(".trae-cn/skills"));
        let direct = specs
            .iter()
            .filter(|tool| tool.reads_shared_center)
            .map(|tool| tool.id)
            .collect::<Vec<_>>();
        assert_eq!(direct, ["kimi-code"]);
    }

    #[test]
    fn tool_root_honours_a_non_empty_override() {
        let home = Path::new("/home/demo");
        assert_eq!(
            resolve_tool_root(home, ".kimi-code", Some("/data/kimi".into())),
            PathBuf::from("/data/kimi")
        );
        assert_eq!(
            resolve_tool_root(home, ".kimi-code", Some("".into())),
            home.join(".kimi-code")
        );
    }

    #[test]
    fn direct_readers_use_the_default_center_without_links() {
        let home = tempfile::tempdir().unwrap();
        let center = home.path().join(".agents/skills");
        let skill = center.join("demo");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join(SKILL_FILE),
            "---\nname: demo\ndescription: Test\n---",
        )
        .unwrap();
        let kimi = find_tool("kimi-code").unwrap();
        let qwen = find_tool("qwen-code").unwrap();
        assert!(reads_center_directly(&kimi, home.path(), &center));
        assert!(!reads_center_directly(&qwen, home.path(), &center));
        let custom = home.path().join("custom-center");
        assert!(!reads_center_directly(&kimi, home.path(), &custom));

        let asset = inspect_skill(
            &center,
            home.path(),
            "demo",
            &skill,
            &TranslationStore::default(),
        );
        let mode = |id: &str| {
            asset
                .connections
                .iter()
                .find(|item| item.tool_id == id)
                .map(|item| item.mode.clone())
        };
        assert_eq!(mode("kimi-code").as_deref(), Some("native"));
        assert_eq!(mode("qwen-code").as_deref(), Some("none"));

        let tools = inspect_tools(home.path(), &center, std::slice::from_ref(&asset));
        let kimi_status = tools.iter().find(|tool| tool.id == "kimi-code").unwrap();
        assert!(kimi_status.reads_center);
        assert_eq!(kimi_status.skills_path, display_path(&center));
        assert_eq!(kimi_status.linked_count, 1);
        assert_eq!(kimi_status.conflict_count, 0);
        assert!(!home.path().join(".kimi-code").exists());
    }

    #[test]
    fn risk_assessment_marks_destructive_commands_high() {
        let (level, flags) = risk_assessment("Run `sudo rm -rf /tmp/example`");
        assert_eq!(level, "high");
        assert!(flags.contains(&"destructive-command".to_owned()));
        assert!(flags.contains(&"privilege-escalation".to_owned()));
    }

    #[test]
    fn readiness_score_has_explainable_weights() {
        assert_eq!(readiness_score(true, true, true, "low", true, true), 100);
        assert_eq!(readiness_score(true, false, true, "high", true, true), 74);
    }

    #[test]
    fn rejects_nested_skill_identifiers() {
        assert!(validate_id("safe-skill").is_ok());
        assert!(validate_id("../unsafe").is_err());
        assert!(validate_id("nested/skill").is_err());
    }
}
