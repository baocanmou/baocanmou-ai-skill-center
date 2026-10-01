import type { Locale, SkillAsset } from './contracts'
import { filterSkills } from './appLogic'

export type SkillGroup = 'main' | 'ppt-styles' | 'ui-styles' | 'advanced'
export type SkillView = SkillGroup | 'all'

// Reviewed presentation groups only: never disable, move, or delete a Skill.
export const reviewedGroups: Record<Exclude<SkillGroup, 'main'>, readonly string[]> = {
  'ppt-styles': [
    'autumn-campus-defense-ppt', 'blue-white-3d-tech-ppt', 'blue-white-minimal-enterprise-ppt',
    'cyan-circuit-launch-ppt', 'dark-neon-tech-business-ppt', 'elegant-chinese-courseware-ppt',
    'gold-orange-business-ppt-infographic', 'green-rural-industry-ppt-infographic',
    'iridescent-lavender-glass-ppt', 'mint-watercolor-business-ppt', 'multicolor-system-architecture-ppt',
    'new-chinese-resume-ppt', 'paper-doodle-courseware-ppt', 'pure-white-handdrawn-ppt-infographic',
    'red-blue-energy-training-ppt', 'red-white-corporate-training-ppt', 'red-white-project-progress-ppt',
    'soft-blue-gold-business-ppt', 'soft-lavender-ppt-infographic', 'teal-education-courseware-ppt',
    'tech-bluegreen-ppt-infographic', 'warm-art-courseware-ppt', 'warm-orange-minimal-ppt-infographic',
  ],
  'ui-styles': [
    'bento', 'brutalist-skill', 'clean', 'corporate', 'editorial', 'elegant', 'glassmorphism',
    'gradient', 'luxury', 'minimal', 'minimalist-skill', 'modern', 'neobrutalism', 'premium',
    'professional', 'shadcn', 'spacious', 'vintage',
  ],
  advanced: [
    'taste-skill-v1', 'interview-me', 'output-skill', 'dbs-chatroom', 'dbs-chatroom-austrian',
    'dbs-slowisfast', 'elon-musk-perspective', 'feynman-perspective', 'steve-jobs-perspective',
    'dbs-agent-migration', 'dbs-bridge', 'dbs-update', 'dbs-save', 'dbs-restore',
    'dbs-skill-cleaner', 'skill-doctor',
  ],
}

export const skillViews: SkillView[] = ['main', 'ppt-styles', 'ui-styles', 'advanced', 'all']

export function skillGroup(id: string): SkillGroup {
  for (const group of ['ppt-styles', 'ui-styles', 'advanced'] as const) {
    if (reviewedGroups[group].includes(id)) return group
  }
  return 'main'
}

export function groupLabel(view: SkillView, locale: Locale): string {
  const labels: Record<SkillView, [string, string]> = {
    main: ['任务技能', 'Task skills'],
    'ppt-styles': ['PPT 配图风格', 'Slide image styles'],
    'ui-styles': ['界面风格', 'UI styles'],
    advanced: ['方法与辅助', 'Methods & utilities'],
    all: ['全部', 'All'],
  }
  return labels[view][locale === 'zh' ? 0 : 1]
}

export function filterSkillView(skills: SkillAsset[], query: string, category: string, view: SkillView): SkillAsset[] {
  // Searching always covers all groups, including compatibility and maintenance entries.
  const candidates = query.trim() || view === 'all' ? skills : skills.filter((skill) => skillGroup(skill.id) === view)
  return filterSkills(candidates, query, category)
}
