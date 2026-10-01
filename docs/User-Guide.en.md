# User Guide · v1.3 Clear edition

Open BaoCanMou AI Skill Center from Applications. It reads `~/.agents/skills` and supported AI-tool paths locally without uploading skill content.

1. Choose one of nine task types on Home, or search by Chinese name, English ID, purpose, or feature.
2. Compare the purpose and features on each card, then open usage.
3. Gather the listed materials, copy the example, and fill in the bracketed fields.
4. Send the request and materials to Codex or Claude. The center does not execute skills itself.

Slide-image styles produce images, not editable PPT files. Choose a presentation workflow when you need an editable deck. Developer previews appear only if supplied by the skill; a missing image is not an execution failure.

All skills remain searchable. Discover contains external references, not automatically installed skills. AI connections shows shared links and independent entries; Checks & notes shows structural and existing static signals. Use Scan again after installing a skill.

Expand Chinese text & editing in the detail view to correct a translation. The app writes only to `~/.baocanmou/skill-center/translations.json`; it does not modify the original Skill. Expand AI connections for link management or Source & technical details for the original instructions. Browsing and copying examples never changes these links.

A filesystem connection does not prove that an already-running AI session loaded the skill, and a structural check does not verify dependencies or accounts. If a skill is not found, start a new AI session and request it by its exact ID, then check the entry and required tools. An independent entry is not necessarily a broken skill.
