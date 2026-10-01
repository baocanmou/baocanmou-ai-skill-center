import { describe, expect, it } from 'vitest'
import type { SkillAsset } from './contracts'
import { kindLabel, skillUsage, starterSkills, taskForSkill, taskGuides, taskIds, taskLabel } from './skillGuide'
import { reviewedGroups } from './skillOrganization'

const fixture = (id: string, category = 'design'): SkillAsset => ({
  id, category, nameZh: '测试技能', nameEn: 'Test skill', summaryZh: '中文说明', summaryEn: 'Description',
  purposeZh: '制作品牌展示图。', purposeEn: 'Create a brand visual.', featuresZh: ['用途清楚'], featuresEn: ['Clear purpose'],
  path: `/test/${id}`, score: 100, status: 'ready', riskLevel: 'low', riskFlags: [], fileCount: 1,
  contentHash: 'unchanged', modifiedAt: 0, translationMode: 'native', previewKind: 'generated', previewCount: 0,
  connections: [{ toolId: 'claude', mode: 'conflict' }],
})

describe('Readable task guides', () => {
  it('provides nine distinct bilingual entries with preparation instructions', () => {
    expect(taskIds).toHaveLength(9)
    expect(new Set(taskIds).size).toBe(9)
    for (const id of taskIds) {
      expect(taskLabel(id, 'zh')).toMatch(/[\u4e00-\u9fff]/)
      expect(taskLabel(id, 'en')).toMatch(/[A-Za-z]/)
      expect(taskGuides[id].inputZh.length).toBeGreaterThan(8)
      expect(taskGuides[id].inputEn.length).toBeGreaterThan(8)
    }
  })

  it('routes presentation and UI presets by purpose rather than broad source categories', () => {
    for (const id of reviewedGroups['ppt-styles']) expect(taskForSkill(fixture(id, 'image'))).toBe('slides')
    for (const id of reviewedGroups['ui-styles']) expect(taskForSkill(fixture(id, 'design'))).toBe('web')
    expect(taskForSkill(fixture('baocanmou-plan-to-ppt', 'presentation'))).toBe('slides')
    expect(taskForSkill(fixture('frontend-design'))).toBe('web')
    expect(taskForSkill(fixture('local-tts', 'content'))).toBe('video')
    expect(taskForSkill(fixture('docling-mcp', 'content'))).toBe('data')
    expect(taskForSkill(fixture('baocanmou-restaurant-slogan', 'marketing'))).toBe('writing')
  })

  it('keeps unknown categories discoverable without altering skills or connections', () => {
    const asset = fixture('unknown', 'new-category')
    const before = structuredClone(asset)
    expect(taskForSkill(asset)).toBe('tools')
    skillUsage(asset, 'zh')
    kindLabel(asset, 'en')
    expect(asset).toEqual(before)
  })

  it('makes image presets distinct from editable presentation workflows', () => {
    const preset = fixture(reviewedGroups['ppt-styles'][0], 'image')
    expect(kindLabel(preset, 'zh')).toBe('PPT 配图风格')
    expect(skillUsage(preset, 'zh').scope).toContain('不是可编辑的 PPT')
    expect(skillUsage(preset, 'en').scope).toContain('not an editable PPT')
  })

  it('creates editable request templates with exact IDs and no claim of successful execution', () => {
    const asset = fixture('logo-generator-skill', 'image')
    const zh = skillUsage(asset, 'zh')
    const en = skillUsage(asset, 'en')
    expect(zh.prompt).toContain(asset.id)
    expect(zh.prompt).toContain(asset.purposeZh)
    expect(zh.prompt).toContain('【填写需求】')
    expect(zh.prompt).toContain('先征得我同意')
    expect(en.prompt).toContain('Ask before spending money')
    expect(en.prompt).toContain(asset.purposeEn)
    expect(zh.scope).toContain('所需账号、接口与软件以技能说明为准')
  })

  it('only shows starter skills actually present in the scanned library', () => {
    const assets = [fixture('unknown'), fixture('logo-generator-skill'), fixture('baocanmou-plan-to-ppt')]
    expect(starterSkills(assets).map(({ id }) => id)).toEqual(['baocanmou-plan-to-ppt', 'logo-generator-skill'])
    expect(starterSkills([])).toEqual([])
  })
})
