import { describe, expect, it } from 'vitest'
import { filterSkillView, groupLabel, reviewedGroups, skillGroup, skillViews } from './skillOrganization'
import type { SkillAsset } from './contracts'

const fixture = (id: string): SkillAsset => ({
  id, nameZh: id, nameEn: id, summaryZh: '', summaryEn: '', purposeZh: '', purposeEn: '',
  featuresZh: [], featuresEn: [], category: 'design', path: `/test/${id}`, score: 100,
  status: 'ready', riskLevel: 'low', riskFlags: [], fileCount: 1, contentHash: '', modifiedAt: 0,
  translationMode: 'native', previewKind: 'generated', previewCount: 0, connections: [],
})

describe('Reviewed catalog organization', () => {
  it('groups exactly 23 slide-image and 18 UI styles without overlapping IDs', () => {
    expect(reviewedGroups['ppt-styles']).toHaveLength(23)
    expect(reviewedGroups['ui-styles']).toHaveLength(18)
    const ids = Object.values(reviewedGroups).flat()
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('keeps production engines and current methods in the main list', () => {
    for (const id of ['guizang-ppt-skill', 'baocanmou-plan-to-ppt', 'taste-skill', 'soft-skill', 'dbs-content-system', 'gsap-core']) {
      expect(skillGroup(id)).toBe('main')
    }
    expect(skillGroup('taste-skill-v1')).toBe('advanced')
    expect(skillGroup('not-reviewed-yet')).toBe('main')
  })

  it('preserves every asset, source path and connection across the groups', () => {
    const skills = ['main-skill', ...Object.values(reviewedGroups).flat()].map(fixture)
    const grouped = skillViews.filter((view) => view !== 'all').flatMap((view) => filterSkillView(skills, '', 'all', view))
    expect(grouped).toHaveLength(skills.length)
    expect(new Set(grouped.map((skill) => skill.id)).size).toBe(skills.length)
    expect(filterSkillView(skills, '', 'all', 'all')).toEqual(skills)
  })

  it('finds old and style skills from the main search without enabling anything', () => {
    const skills = ['taste-skill-v1', 'minimal', 'main-skill'].map(fixture)
    expect(filterSkillView(skills, '', 'all', 'main').map((skill) => skill.id)).toEqual(['main-skill'])
    expect(filterSkillView(skills, 'taste-skill-v1', 'all', 'main')).toEqual([skills[0]])
    expect(filterSkillView(skills, 'minimal', 'all', 'main')).toEqual([skills[1]])
    expect(groupLabel('ppt-styles', 'zh')).toContain('配图')
    expect(groupLabel('ppt-styles', 'en')).toContain('image')
  })
})
