# Changelog

## [1.3.0] - 2026-09-30

- Replaced the metric-led home page with nine task-based entry points and six common-work shortcuts.
- Added preparation guidance, scope notes, and bilingual copyable request templates to every skill.
- Increased text sizes, simplified cards, and moved source text and connection details into expandable sections.
- Kept slide-image styles separate from editable presentation workflows; searches still include all groups.
- Reviewed the Chinese display layer: refreshed 13 source-bound descriptions and added the Baidu keyword skill. Both the initial 234-skill catalog and the current 113-skill catalog pass bilingual name, purpose, and feature validation. The reduction was a separately authorized cleanup, not part of this interface upgrade.
- Preserved skill sources, existing links, and Claude settings. Static checks and filesystem links are not claims of successful skill execution.

## [1.2.0] - 2026-09-11

### Added

- Added dedicated bilingual names and purpose copy for app monetization and last-30-day trend research.
- Added three BaoCanMou-authored adapters: Easel social-content workbench, OmniVoice multilingual TTS, and a catalog-only FreePEP textbook auditor.
- Connected the three adapters to Codex, Claude Code, Hermes, Gemini CLI, Cursor, and ZCode through the shared source.

### Verified

- The center reads 193 local capability assets and returns all five new or updated entries with explicit Chinese and English purposes.
- Easel's 112 upstream Skills, frontend build, and ten safe local tests pass; credentials, gateway, account login, and real publishing remain unconfigured.
- OmniVoice's clean Python environment passes package, CLI, text, and audio-resampling checks; all 13 model files downloaded and a real CPU Chinese inference produced a decodable 3.2-second, 24 kHz mono WAV. Subjective listening approval remains separate.
- FreePEP's local 780-entry catalog and all 31 grade-four first-semester records pass structural checks; all 31 official reading entry points returned HTTP 200.

### Safety

- The FreePEP adapter does not run WAF/CAPTCHA bypass, bulk textbook scraping, PDF assembly, or textbook redistribution.
- The Easel adapter requires a separate approval for login or publishing, and the OmniVoice adapter requires consent for voice cloning.

## [1.0.0] - 2026-09-01

### Added

- Original BaoCanMou Fangce Five-Loop governance core.
- Bilingual interface and editable Chinese understanding for every local Skill.
- Clear purpose and feature labels for every capability.
- Real local screenshot preview with an explicitly generated fallback card.
- Read-only JSON inventory mode for acceptance and automation.

### Changed

- Replaced the previous database, installer, sync engine, and interface modules.
- Changed external recommendations to an index-only, manual-review model.
- Reduced permissions and dependencies to a local-first minimum.

### Removed

- Automatic external Skill installation.
- Remote Skill-content handling.
- Legacy project documents and historical interface assets from the current source tree.

### Security

- Added path-containment checks, file-size limits, managed-target protection, and static risk signals.
