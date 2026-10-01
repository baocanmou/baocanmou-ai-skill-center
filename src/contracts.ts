export type Locale = 'zh' | 'en'

export interface SkillConnection {
  toolId: string
  mode: 'none' | 'link' | 'copy' | 'broken' | 'conflict'
}

export interface SkillAsset {
  id: string
  nameZh: string
  nameEn: string
  summaryZh: string
  summaryEn: string
  purposeZh: string
  purposeEn: string
  featuresZh: string[]
  featuresEn: string[]
  category: string
  path: string
  score: number
  status: 'ready' | 'attention' | 'invalid'
  riskLevel: 'low' | 'medium' | 'high'
  riskFlags: string[]
  fileCount: number
  contentHash: string
  modifiedAt: number
  translationMode: 'native' | 'curated' | 'generated' | 'custom' | 'pending'
  previewKind: 'screenshot' | 'generated'
  previewCount: number
  connections: SkillConnection[]
}

export interface ToolStatus {
  id: string
  name: string
  detected: boolean
  skillsPath: string
  linkedCount: number
  conflictCount: number
}

export interface CenterSnapshot {
  centerPath: string
  generatedAt: number
  skills: SkillAsset[]
  tools: ToolStatus[]
  summary: {
    assetCount: number
    readyCount: number
    attentionCount: number
    connectionCount: number
    chineseReadyCount: number
    screenshotCount: number
  }
}

export interface SkillContent {
  skillId: string
  path: string
  markdown: string
  previewImages: SkillPreviewImage[]
  previewKind: 'screenshot' | 'generated'
}

export interface SkillPreviewImage {
  dataUrl: string
  fileName: string
  label: string
}

export interface CatalogEntry {
  rank: number
  slug: string
  name: string
  summary: string
  downloads: number
  recent_downloads: number
  stars: number
  forks: number
  recommendation_score: number
  recommendation_tier: string
  recommendation_reasons: string[]
  review_flags: string[]
  official: boolean
  category: string
  source: string
  source_url: string
  skills_sh_url: string
  updated_at: string
  license: string
}

export interface CatalogDocument {
  updated_at: string
  total: number
  skills: CatalogEntry[]
}
