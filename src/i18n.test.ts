import { describe, expect, it } from 'vitest'
import { version } from '../package.json'
import { getCopy } from './i18n'

describe('Skill catalog labels', () => {
  it('uses the actual package version in both languages', () => {
    expect(getCopy('zh').version).toContain(`v${version}`)
    expect(getCopy('en').version).toContain(`v${version}`)
  })

  it('does not present structural checks as end-to-end availability', () => {
    expect(getCopy('zh').ready).toBe('结构通过')
    expect(getCopy('en').ready).toBe('Structure checked')
    expect(getCopy('zh').englishOriginal).toBe('英文说明')
  })

  it('distinguishes preserved entries, broken links, and missing previews', () => {
    expect(getCopy('zh').unmanaged).toBe('已有独立入口（未覆盖）')
    expect(getCopy('zh').brokenLink).toContain('链接失效')
    expect(getCopy('zh').previewShortfall).toContain('不代表技能失效')
    expect(getCopy('zh').connections).toBe('共享链接')
    expect(getCopy('en').unmanaged).toBe('Independent entry preserved')
    expect(getCopy('en').brokenLink).toContain('Broken link')
    expect(getCopy('en').previewShortfall).toContain('not a skill failure')
    expect(getCopy('zh').toolsDesc).toContain('Codex 可直接发现共享库')
  })

  it('distinguishes reviewed, upstream and unreviewed Chinese explanations', () => {
    expect(getCopy('zh').translationModes.curated).toContain('核对')
    expect(getCopy('zh').translationModes.generated).toContain('待核对')
    expect(getCopy('zh').translationModes.pending).toContain('待核对')
    expect(getCopy('en').pptStyleNote).toContain('not editable')
  })
})
