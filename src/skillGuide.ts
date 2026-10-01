import type { Locale, SkillAsset } from './contracts'
import { skillGroup } from './skillOrganization'

// Presentation only: these task routes never enable or modify a Skill.
export const taskGuides = {
  brand: { zh: '品牌与营销', en: 'Brand & marketing', hintZh: '品牌策划、推广、SEO / GEO', hintEn: 'Strategy, campaigns, SEO / GEO', inputZh: '品牌或产品、目标客户、要解决的问题', inputEn: 'Brand or product, audience, and objective' },
  writing: { zh: '写文章与文案', en: 'Writing & content', hintZh: '公众号、社媒、广告语', hintEn: 'Articles, social posts, copy', inputZh: '主题、发布平台、读者和参考资料', inputEn: 'Topic, channel, readers, and reference material' },
  slides: { zh: '做 PPT', en: 'Presentations', hintZh: '提案、汇报、演示配图', hintEn: 'Proposals, reports, slide visuals', inputZh: '现有资料、给谁看、页数和交付格式', inputEn: 'Source material, audience, slide count, and format' },
  visuals: { zh: '设计与画图', en: 'Design & images', hintZh: 'Logo、插画、电商图', hintEn: 'Logos, illustrations, product images', inputZh: '画面主题、参考图、尺寸和使用场景', inputEn: 'Subject, reference images, dimensions, and intended use' },
  video: { zh: '视频与配音', en: 'Video & audio', hintZh: '剪辑、动效、旁白', hintEn: 'Editing, motion, narration', inputZh: '文案或素材、时长、画幅和声音要求', inputEn: 'Script or footage, duration, aspect ratio, and audio needs' },
  web: { zh: '网站与开发', en: 'Web & development', hintZh: '页面、代码、服务器', hintEn: 'Interfaces, code, servers', inputZh: '项目或网址、具体需求、现有技术环境', inputEn: 'Project or URL, requirements, and existing environment' },
  research: { zh: '查资料与研究', en: 'Research', hintZh: '搜索、竞品、趋势', hintEn: 'Search, competitors, trends', inputZh: '研究问题、范围、时间要求和已有资料', inputEn: 'Research question, scope, timeframe, and sources' },
  data: { zh: '文档与数据', en: 'Documents & data', hintZh: '文件提取、表格、分析', hintEn: 'Extraction, tables, analysis', inputZh: '原始文件、要处理的内容和输出格式', inputEn: 'Source files, processing requirements, and output format' },
  tools: { zh: 'AI 与日常工具', en: 'AI & utilities', hintZh: '浏览器、知识库、工作方法', hintEn: 'Browser, knowledge base, workflows', inputZh: '正在用的软件、任务目标和当前遇到的问题', inputEn: 'Software in use, objective, and current problem' },
} as const

export type TaskId = keyof typeof taskGuides
export const taskIds = Object.keys(taskGuides) as TaskId[]

export function taskForSkill(skill: Pick<SkillAsset, 'id' | 'category'>): TaskId {
  const group = skillGroup(skill.id)
  if (group === 'ppt-styles' || skill.category === 'presentation') return 'slides'
  if (group === 'ui-styles') return 'web'
  if (/(?:^|[-_])(tts|audio)(?:[-_]|$)/.test(skill.id) || skill.category === 'video') return 'video'
  if (['frontend-design', 'taste-skill', 'taste-skill-v1', 'soft-skill', 'gpt-tasteskill', 'impeccable', 'image-to-code-skill', 'imagegen-frontend-web', 'imagegen-frontend-mobile', 'open-design'].includes(skill.id)) return 'web'
  if (['markitdown', 'docling-mcp', 'defuddle'].includes(skill.id)) return 'data'
  if (['baocanmou-wordpress-esa-trust-release', 'opencli-browser-sitemap'].includes(skill.id)) return 'web'
  if (skill.id === 'baocanmou-restaurant-slogan') return 'writing'
  if (skill.category === 'marketing') return 'brand'
  if (skill.category === 'content') return 'writing'
  if (['image', 'design'].includes(skill.category)) return 'visuals'
  if (['development', 'devops', 'security'].includes(skill.category)) return 'web'
  if (skill.category === 'research') return 'research'
  if (['data', 'database', 'data-visualization'].includes(skill.category)) return 'data'
  return 'tools'
}

export function taskLabel(task: TaskId, locale: Locale): string {
  return taskGuides[task][locale]
}

export function kindLabel(skill: SkillAsset, locale: Locale): string {
  const group = skillGroup(skill.id)
  const labels = {
    main: ['任务技能', 'Task skill'],
    'ppt-styles': ['PPT 配图风格', 'Slide image style'],
    'ui-styles': ['界面风格', 'UI style'],
    advanced: ['方法与辅助', 'Method & utility'],
  }
  return labels[group][locale === 'zh' ? 0 : 1]
}

export function skillUsage(skill: SkillAsset, locale: Locale) {
  const task = taskGuides[taskForSkill(skill)]
  const group = skillGroup(skill.id)
  const zh = locale === 'zh'
  const name = zh ? skill.nameZh : skill.nameEn
  const purpose = zh ? skill.purposeZh : skill.purposeEn
  const prepare = task[zh ? 'inputZh' : 'inputEn']
  const scope = group === 'ppt-styles'
    ? (zh ? '这是演示配图风格，输出图片，不是可编辑的 PPT 文件。' : 'This is a slide-image style. It produces images, not an editable PPT file.')
    : group === 'ui-styles'
      ? (zh ? '这是界面视觉风格；请附上要调整的页面或项目。' : 'This is a UI style. Provide the page or project to adapt.')
      : (zh ? '在 Codex 或 Claude 中发送下方示例；所需账号、接口与软件以技能说明为准。' : 'Send the example in Codex or Claude. Accounts, APIs, and software requirements depend on the skill.')
  const prompt = zh
    ? `请使用技能 ${skill.id}（${name}）。\n目标：${purpose}\n我的具体需求：【填写需求】\n提供的资料：【${prepare}】\n请先确认适用范围和所需工具，再开始；涉及付费、发布或修改线上内容时先征得我同意。`
    : `Use the ${skill.id} skill (${name}).\nGoal: ${purpose}\nMy request: [describe the task]\nMaterials: [${prepare}]\nCheck the scope and required tools first. Ask before spending money, publishing, or changing live content.`
  return { prepare, scope, prompt }
}

const starterIds = ['baocanmou-plan-to-ppt', 'logo-generator-skill', 'baocanmou-content-matrix', 'gzh-design', 'bcm-geo-optimizer', 'baocut']
export function starterSkills(skills: SkillAsset[]): SkillAsset[] {
  return starterIds.flatMap((id) => skills.filter((skill) => skill.id === id))
}
