import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  Activity,
  ArrowUpRight,
  BookOpenText,
  Check,
  Copy,
  ChevronRight,
  CircleAlert,
  Database,
  Globe2,
  Images,
  Languages,
  Link2,
  LoaderCircle,
  RefreshCw,
  Search,
  ShieldCheck,
  Unlink,
  X,
} from 'lucide-react'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import catalogDocument from '../featured-skills.json'
import {
  categoryLabel,
  catalogChineseName,
  catalogPurpose,
  filterCatalog,
  formatCompactNumber,
  recommendationReason,
} from './appLogic'
import type {
  CatalogDocument,
  CatalogEntry,
  CenterSnapshot,
  Locale,
  SkillAsset,
  SkillContent,
  ToolStatus,
} from './contracts'
import { getCopy } from './i18n'
import { filterSkillView, groupLabel, skillGroup, skillViews, type SkillView } from './skillOrganization'
import { kindLabel, skillUsage, starterSkills, taskForSkill, taskGuides, taskIds, taskLabel, type TaskId } from './skillGuide'
import brandMark from './assets/baocanmou-mark.svg'

type Section = 'command' | 'assets' | 'intelligence' | 'tools' | 'audit' | 'about'

const catalog = catalogDocument as CatalogDocument

const sections: Section[] = ['command', 'assets', 'intelligence', 'tools', 'audit', 'about']

function isDesktopRuntime(): boolean {
  return '__TAURI_INTERNALS__' in window
}

function demoSnapshot(): CenterSnapshot {
  const skills = catalog.skills.slice(0, 12).map<SkillAsset>((item) => ({
    id: item.name,
    nameZh: catalogChineseName(item),
    nameEn: item.name,
    summaryZh: catalogPurpose(item, 'zh'),
    summaryEn: item.summary || 'Browser preview data.',
    purposeZh: catalogPurpose(item, 'zh'),
    purposeEn: catalogPurpose(item, 'en'),
    featuresZh: item.recommendation_reasons.slice(0, 3).map((reason) => recommendationReason(reason, 'zh')),
    featuresEn: item.recommendation_reasons.slice(0, 3).map((reason) => recommendationReason(reason, 'en')),
    category: item.category,
    path: `~/.agents/skills/${item.name}`,
    score: Math.round(item.recommendation_score),
    status: 'ready',
    riskLevel: 'low',
    riskFlags: [],
    fileCount: 1,
    contentHash: item.slug,
    modifiedAt: 0,
    translationMode: 'generated',
    previewKind: 'generated',
    previewCount: 0,
    connections: [],
  }))
  return {
    centerPath: '~/.agents/skills',
    generatedAt: Math.floor(Date.now() / 1000),
    skills,
    tools: ['Codex', 'Claude Code', 'Gemini CLI', 'Cursor'].map<ToolStatus>((name, index) => ({
      id: name.toLowerCase().replaceAll(' ', '-'),
      name,
      detected: index < 2,
      skillsPath: `~/.${name.toLowerCase().split(' ')[0]}/skills`,
      linkedCount: index < 2 ? skills.length : 0,
      conflictCount: 0,
    })),
    summary: {
      assetCount: skills.length,
      readyCount: skills.length,
      attentionCount: 0,
      connectionCount: skills.length * 2,
      chineseReadyCount: skills.length,
      screenshotCount: 0,
    },
  }
}

export default function App() {
  const [locale, setLocale] = useState<Locale>(() => (localStorage.getItem('bcm-locale') === 'en' ? 'en' : 'zh'))
  const [section, setSection] = useState<Section>('command')
  const [snapshot, setSnapshot] = useState<CenterSnapshot | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [query, setQuery] = useState('')
  const [category, setCategory] = useState('all')
  const [task, setTask] = useState<TaskId | 'all'>('all')
  const [selectedSkill, setSelectedSkill] = useState<SkillAsset | null>(null)
  const [workingKey, setWorkingKey] = useState('')
  const t = getCopy(locale)

  const scan = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      setSnapshot(isDesktopRuntime() ? await invoke<CenterSnapshot>('scan_center') : demoSnapshot())
    } catch (scanError) {
      setError(String(scanError))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void scan()
  }, [scan])

  useEffect(() => {
    localStorage.setItem('bcm-locale', locale)
    document.documentElement.lang = locale === 'zh' ? 'zh-CN' : 'en'
  }, [locale])

  const chooseSection = (next: Section) => {
    setSection(next)
    setQuery('')
    setCategory('all')
    setTask('all')
  }

  const updateSnapshot = (next: CenterSnapshot, selectedId?: string) => {
    setSnapshot(next)
    if (selectedId) setSelectedSkill(next.skills.find((skill) => skill.id === selectedId) ?? null)
  }

  const handleConnection = async (skillId: string, toolId: string, connected: boolean) => {
    if (!isDesktopRuntime()) return
    const key = `${skillId}:${toolId}`
    setWorkingKey(key)
    setError('')
    try {
      const command = connected ? 'disconnect_skill' : 'connect_skill'
      const next = await invoke<CenterSnapshot>(command, { skillId, toolId })
      updateSnapshot(next, skillId)
    } catch (actionError) {
      setError(String(actionError))
    } finally {
      setWorkingKey('')
    }
  }

  const handleSaveTranslation = async (skillId: string, nameZh: string, summaryZh: string) => {
    if (!isDesktopRuntime()) return
    setWorkingKey(`translate:${skillId}`)
    setError('')
    try {
      const next = await invoke<CenterSnapshot>('save_translation', {
        input: { skillId, nameZh, summaryZh },
      })
      updateSnapshot(next, skillId)
    } catch (actionError) {
      setError(String(actionError))
    } finally {
      setWorkingKey('')
    }
  }

  const handleExternal = async (url: string) => {
    if (isDesktopRuntime()) await openUrl(url)
    else window.open(url, '_blank', 'noopener,noreferrer')
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="window-drag" data-tauri-drag-region />
        <div className="brand-lockup">
          <img src={brandMark} alt="" />
          <div>
            <strong>{t.brand}</strong>
            <span>{t.version}</span>
          </div>
        </div>
        <nav className="primary-nav" aria-label="Primary">
          {sections.map((item, index) => {
            return (
              <button key={item} aria-label={t.nav[index]} title={t.nav[index]} aria-current={section === item ? 'page' : undefined} className={section === item ? 'active' : ''} onClick={() => chooseSection(item)}>
                <b className="nav-index">{String(index + 1).padStart(2, '0')}</b>
                <span><strong>{t.nav[index]}</strong><small>{t.navHint[index]}</small></span>
                <ChevronRight size={15} />
              </button>
            )
          })}
        </nav>
        <div className="sidebar-foot">
          <span className="status-dot" />
          <div>
            <strong>{t.localOnly}</strong>
            <small>{snapshot?.centerPath ?? '~/.agents/skills'}</small>
          </div>
        </div>
      </aside>

      <main className="main-panel">
        <header className="topbar" data-tauri-drag-region>
          <div className="crumb">
            <span>BAOCANMOU</span>
            <ChevronRight size={14} />
            <strong>{t.nav[sections.indexOf(section)]}</strong>
          </div>
          <div className="top-actions">
            <button className="language-toggle" onClick={() => setLocale(locale === 'zh' ? 'en' : 'zh')}>
              <Languages size={16} /> {locale === 'zh' ? 'EN' : '中文'}
            </button>
            <button className="scan-button" onClick={() => void scan()} disabled={loading}>
              <RefreshCw size={16} className={loading ? 'spin' : ''} /> {t.refresh}
            </button>
          </div>
        </header>

        <div className="content-scroll">
          {!isDesktopRuntime() && <div className="preview-banner">{t.browserPreview}</div>}
          {error && <div className="error-banner"><CircleAlert size={17} /> <span>{t.error}：{error}</span></div>}
          {loading && !snapshot ? <Loading copy={t.loading} /> : null}
          {snapshot && section === 'command' && <CommandPage snapshot={snapshot} locale={locale} onSelect={setSelectedSkill} onSearch={(value) => { chooseSection('assets'); setQuery(value) }} onTask={(value) => { chooseSection('assets'); setTask(value) }} />}
          {snapshot && section === 'assets' && (
            <AssetsPage
              snapshot={snapshot}
              locale={locale}
              query={query}
              category={category}
              onQuery={setQuery}
              onCategory={setCategory}
              onSelect={setSelectedSkill}
              task={task}
              onTask={setTask}
            />
          )}
          {section === 'intelligence' && (
            <IntelligencePage locale={locale} query={query} category={category} onQuery={setQuery} onCategory={setCategory} onOpen={handleExternal} />
          )}
          {snapshot && section === 'tools' && <ToolsPage snapshot={snapshot} locale={locale} />}
          {snapshot && section === 'audit' && <AuditPage snapshot={snapshot} locale={locale} onSelect={setSelectedSkill} />}
          {section === 'about' && <AboutPage locale={locale} onOpen={handleExternal} />}
        </div>
      </main>

      {selectedSkill && snapshot && (
        <SkillDrawer
          key={selectedSkill.id}
          skill={selectedSkill}
          tools={snapshot.tools}
          locale={locale}
          workingKey={workingKey}
          onClose={() => setSelectedSkill(null)}
          onConnection={handleConnection}
          onSaveTranslation={handleSaveTranslation}
        />
      )}
    </div>
  )
}

function Loading({ copy }: { copy: string }) {
  return <div className="loading-state"><LoaderCircle className="spin" size={24} /><span>{copy}</span></div>
}

function CommandPage({ snapshot, locale, onSelect, onSearch, onTask }: {
  snapshot: CenterSnapshot; locale: Locale; onSelect: (skill: SkillAsset) => void
  onSearch: (query: string) => void; onTask: (task: TaskId | 'all') => void
}) {
  const t = getCopy(locale)
  const [search, setSearch] = useState('')
  const starters = starterSkills(snapshot.skills)
  return (
    <div className="page command-page">
      <section className="welcome">
        <div><span className="eyebrow">BAOCANMOU SKILLS</span><h1>{t.welcome}</h1><p>{t.welcomeHint}</p></div>
        <span className="library-count">{snapshot.skills.length}<small>{t.localSkills}</small></span>
      </section>
      <form className="home-search" onSubmit={(event) => { event.preventDefault(); onSearch(search) }}>
        <SearchBar locale={locale} value={search} onChange={setSearch} />
        <button type="submit">{t.findSkill}<ChevronRight size={16} /></button>
      </form>
      <div className="task-grid">
        {taskIds.map((id, index) => {
          const count = snapshot.skills.filter((skill) => taskForSkill(skill) === id).length
          return <button key={id} onClick={() => onTask(id)}>
            <span className="task-number">{String(index + 1).padStart(2, '0')}</span>
            <strong>{taskLabel(id, locale)}</strong>
            <small>{taskGuides[id][locale === 'zh' ? 'hintZh' : 'hintEn']}</small>
            <span className="task-count">{count} {t.items}<ChevronRight size={14} /></span>
          </button>
        })}
      </div>
      <section className="panel recent-panel">
        <div className="section-heading"><div><h2>{t.startHere}</h2><p>{t.startHereHint}</p></div><button className="text-button" onClick={() => onTask('all')}>{t.browseAll}<ChevronRight size={15} /></button></div>
        <div className="recent-list">
          {starters.map((skill) => <SkillRow key={skill.id} skill={skill} locale={locale} onSelect={() => onSelect(skill)} />)}
        </div>
      </section>
      <div className="start-help"><span>01 {t.stepChoose}</span><ChevronRight size={14} /><span>02 {t.stepCopy}</span><ChevronRight size={14} /><span>03 {t.stepSend}</span></div>
      <p className="library-footnote">{t.localOnly} · {t.chinese} {snapshot.summary.chineseReadyCount}/{snapshot.skills.length} · {t.refreshHint}</p>
    </div>
  )
}

function SearchBar({ locale, value, onChange }: { locale: Locale; value: string; onChange: (value: string) => void }) {
  const t = getCopy(locale)
  return <label className="search-field"><Search size={17} /><input aria-label={t.search} value={value} onChange={(event) => onChange(event.target.value)} placeholder={t.search} /></label>
}

function CategoryTabs({ values, current, locale, onChange }: { values: string[]; current: string; locale: Locale; onChange: (value: string) => void }) {
  const t = getCopy(locale)
  return (
    <div className="category-tabs">
      {['all', ...values].map((value) => (
        <button key={value} className={current === value ? 'active' : ''} onClick={() => onChange(value)}>
          {value === 'all' ? t.all : categoryLabel(value, locale)}
        </button>
      ))}
    </div>
  )
}

function AssetsPage({ snapshot, locale, query, category, onQuery, onCategory, onSelect, task, onTask }: {
  snapshot: CenterSnapshot
  locale: Locale
  query: string
  category: string
  onQuery: (value: string) => void
  onCategory: (value: string) => void
  onSelect: (skill: SkillAsset) => void
  task: TaskId | 'all'
  onTask: (task: TaskId | 'all') => void
}) {
  const t = getCopy(locale)
  const [view, setView] = useState<SkillView>('all')
  const activeView = query.trim() ? 'all' : view
  const skills = useMemo(() => filterSkillView(snapshot.skills, query, category, view).filter((skill) => query.trim() || task === 'all' || taskForSkill(skill) === task), [snapshot.skills, query, category, view, task])
  const viewCounts = Object.fromEntries(skillViews.map((item) => [item, snapshot.skills.filter((skill) => (item === 'all' || skillGroup(skill.id) === item) && (task === 'all' || taskForSkill(skill) === task)).length]))
  return (
    <div className="page">
      <div className="page-title"><div><h1>{t.assets}</h1><p>{t.libraryHint}</p></div><strong>{snapshot.skills.length}</strong></div>
      <SearchBar locale={locale} value={query} onChange={(value) => { onQuery(value); onCategory('all'); onTask('all') }} />
      <div className="task-tabs" role="group" aria-label={t.byTask}>
        {(['all', ...taskIds] as const).map((id) => <button key={id} aria-pressed={task === id} className={task === id ? 'active' : ''} onClick={() => { onTask(id); onQuery(''); setView('all'); onCategory('all') }}>{id === 'all' ? t.all : taskLabel(id, locale)}</button>)}
      </div>
      <div className="skill-view-tabs" role="group" aria-label={t.assetGroups}>
        {skillViews.filter((item) => item === 'all' || viewCounts[item] > 0).map((item) => (
          <button key={item} aria-pressed={activeView === item} className={activeView === item ? 'active' : ''} onClick={() => { setView(item); onQuery(''); onCategory('all') }}>
            {groupLabel(item, locale)}<span>{viewCounts[item]}</span>
          </button>
        ))}
      </div>
      <p className="asset-view-note" role="status">
        {skills.length} {t.items} · {query.trim() ? t.searchAllGroups : activeView === 'ppt-styles' ? t.pptStyleNote : activeView === 'advanced' ? t.advancedNote : t.clickForUsage}
      </p>
      <div className="asset-grid">
        {skills.map((skill) => <SkillCard key={skill.id} skill={skill} locale={locale} onSelect={() => onSelect(skill)} />)}
      </div>
      {!skills.length && <div className="empty-state">{t.empty}</div>}
    </div>
  )
}

function SkillCard({ skill, locale, onSelect }: { skill: SkillAsset; locale: Locale; onSelect: () => void }) {
  const t = getCopy(locale)
  const features = locale === 'zh' ? skill.featuresZh : skill.featuresEn
  const glyph = categoryLabel(skill.category, 'zh').slice(0, 1)
  return (
    <article className="skill-card" onClick={onSelect}>
      <div className="skill-card-heading">
        <span className={`skill-card-icon category-${skill.category}`}>{glyph}</span>
        <div><h3>{locale === 'zh' ? skill.nameZh : skill.nameEn}</h3><code>{skill.id}</code></div>
      </div>
      <div className="skill-purpose">
        <span>{t.purpose}</span>
        <p>{locale === 'zh' ? skill.purposeZh : skill.purposeEn}</p>
      </div>
      <div className="skill-card-features">
        {features.slice(0, 3).map((feature) => <span key={feature}><Check size={11} />{feature}</span>)}
      </div>
      <div className="skill-meta">
        <span>{kindLabel(skill, locale)}</span>
        {skill.previewCount > 0 && <span className="preview-count">{skill.previewCount} {t.visualExamples}</span>}
      </div>
      <div className="skill-card-footer"><span>{skill.translationMode === 'pending' ? t.translationModes.pending : taskLabel(taskForSkill(skill), locale)}</span><button onClick={(event) => { event.stopPropagation(); onSelect() }}>{t.details}<ChevronRight size={15} /></button></div>
    </article>
  )
}

function SkillRow({ skill, locale, onSelect }: { skill: SkillAsset; locale: Locale; onSelect: () => void }) {
  return (
    <button className="skill-row" onClick={onSelect}>
      <span className="row-glyph">{categoryLabel(skill.category, 'zh').slice(0, 1)}</span>
      <span><strong>{locale === 'zh' ? skill.nameZh : skill.nameEn}</strong><small>{locale === 'zh' ? skill.purposeZh : skill.purposeEn}</small></span>
      <em>{kindLabel(skill, locale)}</em>
      <ChevronRight size={16} />
    </button>
  )
}

function IntelligencePage({ locale, query, category, onQuery, onCategory, onOpen }: {
  locale: Locale
  query: string
  category: string
  onQuery: (value: string) => void
  onCategory: (value: string) => void
  onOpen: (url: string) => Promise<void>
}) {
  const t = getCopy(locale)
  const categories = useMemo(() => [...new Set(catalog.skills.map((skill) => skill.category))].sort(), [])
  const skills = useMemo(() => filterCatalog(catalog.skills, query, category), [query, category])
  return (
    <div className="page">
      <div className="page-title"><div><span className="eyebrow">EXTERNAL INDEX · NOT BUNDLED</span><h1>{t.intelligenceTitle}</h1><p>{t.intelligenceDesc}</p></div><strong>{catalog.total}</strong></div>
      <SearchBar locale={locale} value={query} onChange={onQuery} />
      <CategoryTabs values={categories} current={category} locale={locale} onChange={onCategory} />
      <div className="intelligence-list">
        {skills.map((skill) => <CatalogRow key={skill.slug} skill={skill} locale={locale} onOpen={onOpen} />)}
      </div>
      {!skills.length && <div className="empty-state">{t.empty}</div>}
    </div>
  )
}

function CatalogRow({ skill, locale, onOpen }: { skill: CatalogEntry; locale: Locale; onOpen: (url: string) => Promise<void> }) {
  const t = getCopy(locale)
  return (
    <article className="catalog-row">
      <span className="rank">{String(skill.rank).padStart(2, '0')}</span>
      <CatalogVisual skill={skill} />
      <div className="catalog-main">
        <div><h3>{locale === 'zh' ? catalogChineseName(skill) : skill.name}</h3><code>{skill.name} · {skill.source}</code></div>
        <p>{catalogPurpose(skill, locale)}</p>
        <div className="reason-list">{skill.recommendation_reasons.slice(0, 4).map((reason) => <span key={reason}>{recommendationReason(reason, locale)}</span>)}</div>
      </div>
      <div className="catalog-numbers">
        <span><b>{formatCompactNumber(skill.downloads, locale)}</b><small>{t.downloads}</small></span>
        <span><b>{formatCompactNumber(skill.stars, locale)}</b><small>{t.stars}</small></span>
        <span className="catalog-score"><b>{skill.recommendation_score.toFixed(1)}</b><small>{t.score}</small></span>
      </div>
      <button onClick={() => void onOpen(skill.source_url)}>{t.source}<ArrowUpRight size={15} /></button>
    </article>
  )
}

function CatalogVisual({ skill }: { skill: CatalogEntry }) {
  return (
    <div className={`catalog-visual category-${skill.category}`}>
      <span>{categoryLabel(skill.category, 'zh').slice(0, 1)}</span>
      <i /><i /><i />
    </div>
  )
}

function ToolsPage({ snapshot, locale }: { snapshot: CenterSnapshot; locale: Locale }) {
  const t = getCopy(locale)
  return (
    <div className="page">
      <div className="page-title"><div><span className="eyebrow">ONE SOURCE · MANY TOOLS</span><h1>{t.toolsTitle}</h1><p>{t.toolsDesc}</p></div><strong>{snapshot.tools.filter((tool) => tool.detected).length}/{snapshot.tools.length}</strong></div>
      <div className="tool-grid">
        {snapshot.tools.map((tool) => (
          <article key={tool.id} className={tool.detected ? 'tool-card detected' : 'tool-card'}>
            <div className="tool-monogram">{tool.name.slice(0, 2).toUpperCase()}</div>
            <div><h3>{tool.name}</h3><code>{tool.skillsPath}</code></div>
            <span className={tool.detected ? 'detected-label' : 'muted-label'}>{tool.detected ? t.detected : t.notDetected}</span>
            <dl><div><dt>{t.connected}</dt><dd>{tool.linkedCount}</dd></div><div><dt>{t.conflict}</dt><dd>{tool.conflictCount}</dd></div></dl>
          </article>
        ))}
      </div>
    </div>
  )
}

function AuditPage({ snapshot, locale, onSelect }: { snapshot: CenterSnapshot; locale: Locale; onSelect: (skill: SkillAsset) => void }) {
  const t = getCopy(locale)
  const risky = [...snapshot.skills].sort((a, b) => {
    const weight = { high: 3, medium: 2, low: 1 }
    return weight[b.riskLevel] - weight[a.riskLevel] || a.score - b.score
  })
  return (
    <div className="page">
      <div className="page-title"><div><span className="eyebrow">STATIC REVIEW</span><h1>{t.auditTitle}</h1><p>{t.auditDesc}</p></div><strong>{snapshot.summary.attentionCount}</strong></div>
      <div className="audit-summary">
        {(['high', 'medium', 'low'] as const).map((level) => <article key={level} className={level}><span>{t[level]}</span><strong>{snapshot.skills.filter((skill) => skill.riskLevel === level).length}</strong></article>)}
      </div>
      <div className="audit-list">
        {risky.map((skill) => (
          <button key={skill.id} onClick={() => onSelect(skill)}>
            <span className={`risk-mark ${skill.riskLevel}`}><Activity size={16} /></span>
            <span><strong>{locale === 'zh' ? skill.nameZh : skill.nameEn}</strong><small>{skill.riskFlags.length ? skill.riskFlags.join(' · ') : (locale === 'zh' ? '未发现静态风险特征' : 'No static risk signal found')}</small></span>
            <em>{skill.score}</em><ChevronRight size={16} />
          </button>
        ))}
      </div>
    </div>
  )
}

function AboutPage({ locale, onOpen }: { locale: Locale; onOpen: (url: string) => Promise<void> }) {
  const t = getCopy(locale)
  return (
    <div className="page about-page">
      <section className="about-hero">
        <img src={brandMark} alt="" />
        <span className="eyebrow">BAOCANMOU · WWW.BCMSJ.COM</span>
        <h1>{t.aboutTitle}</h1>
        <p>{t.aboutBody}</p>
      </section>
      <section className="originality-grid">
        {[
          [Database, locale === 'zh' ? '自有数据模型' : 'Original data model', locale === 'zh' ? '不依赖旧项目数据库与同步引擎。' : 'Independent from legacy project databases and sync engines.'],
          [Languages, locale === 'zh' ? '中文理解层' : 'Chinese understanding', locale === 'zh' ? '保留英文兼容标识，生成并允许校正中文名称与说明。' : 'Preserve compatible IDs while generating editable Chinese names and summaries.'],
          [Activity, locale === 'zh' ? '方策评分' : 'Fangce scoring', locale === 'zh' ? '结构、理解、可移植、安全、可核验五维计分。' : 'Structure, understanding, portability, safety, and verifiability.'],
          [ShieldCheck, locale === 'zh' ? '本地安全边界' : 'Local safety boundary', locale === 'zh' ? '不自动安装，不上传内容，不覆盖非托管目录。' : 'No automatic installs, uploads, or unmanaged overwrites.'],
        ].map(([Icon, title, body]) => (
          <article key={String(title)}><Icon size={22} /><h3>{String(title)}</h3><p>{String(body)}</p></article>
        ))}
      </section>
      <section className="license-panel"><BookOpenText size={20} /><div><strong>MIT OPEN SOURCE</strong><p>{t.openSource}</p></div></section>
      <div className="about-actions">
        <button onClick={() => void onOpen('https://www.bcmsj.com')}>{t.website}<Globe2 size={16} /></button>
        <button onClick={() => void onOpen('https://github.com/yht0912/baocanmou-ai-skill-center')}>{t.repository}<ArrowUpRight size={16} /></button>
      </div>
    </div>
  )
}

function SkillDrawer({ skill, tools, locale, workingKey, onClose, onConnection, onSaveTranslation }: {
  skill: SkillAsset
  tools: ToolStatus[]
  locale: Locale
  workingKey: string
  onClose: () => void
  onConnection: (skillId: string, toolId: string, connected: boolean) => Promise<void>
  onSaveTranslation: (skillId: string, nameZh: string, summaryZh: string) => Promise<void>
}) {
  const t = getCopy(locale)
  const [content, setContent] = useState<SkillContent | null>(null)
  const [contentError, setContentError] = useState('')
  const [editing, setEditing] = useState(false)
  const [nameZh, setNameZh] = useState(skill.nameZh)
  const [summaryZh, setSummaryZh] = useState(skill.summaryZh)
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'failed'>('idle')
  const closeRef = useRef<HTMLButtonElement>(null)
  const drawerRef = useRef<HTMLElement>(null)
  const usage = skillUsage(skill, locale)

  async function copyPrompt() {
    try {
      await navigator.clipboard.writeText(usage.prompt)
      setCopyState('copied')
    } catch {
      setCopyState('failed')
    }
  }

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null
    closeRef.current?.focus()
    return () => previousFocus?.focus()
  }, [])

  useEffect(() => {
    // WebKit can return focus to the document after a native clipboard action.
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }
    document.addEventListener('keydown', closeOnEscape)
    return () => document.removeEventListener('keydown', closeOnEscape)
  }, [onClose])

  useEffect(() => {
    if (!isDesktopRuntime()) return
    let active = true
    void invoke<SkillContent>('read_skill', { skillId: skill.id })
      .then((value) => { if (active) setContent(value) })
      .catch((value: unknown) => { if (active) setContentError(String(value)) })
    return () => { active = false }
  }, [skill.id])

  return (
    <div className="drawer-layer" role="dialog" aria-modal="true" aria-labelledby="skill-detail-title" onKeyDown={(event) => {
      if (event.key !== 'Tab') return
      const focusable = [...(drawerRef.current?.querySelectorAll<HTMLElement>('button:not(:disabled), input, textarea, summary, a[href]') ?? [])].filter((item) => item.offsetParent !== null)
      const first = focusable[0]
      const last = focusable.at(-1)
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus() }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus() }
    }}>
      <button className="drawer-backdrop" tabIndex={-1} aria-label={t.cancel} onClick={onClose} />
      <aside className="skill-drawer" ref={drawerRef}>
        <div className="drawer-header"><button ref={closeRef} aria-label={t.back} onClick={onClose}><X size={18} /></button><span>{skill.id}</span></div>
        <div className="drawer-scroll">
          <div className="drawer-title"><span className="eyebrow">{kindLabel(skill, locale)} · {taskLabel(taskForSkill(skill), locale)}</span><h2 id="skill-detail-title">{locale === 'zh' ? skill.nameZh : skill.nameEn}</h2></div>

          <section className="drawer-section">
            <div className="drawer-section-title"><strong>{t.purpose}</strong></div>
            <p className="purpose-copy">{locale === 'zh' ? skill.purposeZh : skill.purposeEn}</p>
            <div className="drawer-section-title features-title"><strong>{t.features}</strong></div>
            <ul className="feature-list">{(locale === 'zh' ? skill.featuresZh : skill.featuresEn).map((feature) => <li key={feature}><Check size={13} />{feature}</li>)}</ul>
          </section>

          <section className="drawer-section usage-section">
            <div className="drawer-section-title"><strong>{t.howToUse}</strong></div>
            <p className="prepare-copy"><b>{t.prepare}</b>{usage.prepare}</p>
            <p className="usage-scope">{usage.scope}</p>
            <label className="prompt-label" htmlFor="skill-prompt">{t.promptExample}</label>
            <textarea id="skill-prompt" className="usage-prompt" value={usage.prompt} readOnly rows={7} />
            <p className="preview-note">{t.templateNote}</p>
            <button className="copy-prompt" onClick={() => void copyPrompt()}><Copy size={16} />{t.copyPrompt}</button>
            <p className={copyState === 'failed' ? 'inline-error' : 'copy-feedback'} role="status">{copyState === 'copied' ? t.copied : copyState === 'failed' ? t.copyFailed : ''}</p>
          </section>

          {(skill.category === 'image' || (content?.previewImages.length ?? 0) > 0) && (
            <section className="drawer-section preview-section">
              <div className="drawer-section-title">
                <strong>{t.preview}</strong>
                <span>{t.realScreenshot} · {content?.previewImages.length ?? 0}/{skill.previewCount}</span>
              </div>
              {contentError ? <p className="inline-error">{contentError}</p> : !content && isDesktopRuntime() ? (
                <div className="source-placeholder"><LoaderCircle className="spin" size={18} /></div>
              ) : content?.previewImages.length ? (
                <div className={`skill-preview-grid count-${content.previewImages.length}`}>
                  {content.previewImages.map((preview, index) => (
                    <figure key={`${preview.fileName}-${index}`}>
                      <div className="preview-frame">
                        <img src={preview.dataUrl} alt={`${locale === 'zh' ? skill.nameZh : skill.nameEn} ${t.preview} ${index + 1}`} />
                        <b>{String(index + 1).padStart(2, '0')}</b>
                      </div>
                      <figcaption>{preview.label}</figcaption>
                    </figure>
                  ))}
                </div>
              ) : (
                <div className="preview-empty"><Images size={22} /><span>{t.previewEmpty}</span></div>
              )}
              {content && (
                <p className="preview-note">
                  {skill.previewCount < 3 ? t.previewShortfall : t.previewPolicy}
                </p>
              )}
            </section>
          )}

          <details className="drawer-section secondary-details">
            <summary>{t.moreDetails}</summary>
            <div className="drawer-section-title"><strong>{t.chineseUnderstanding}</strong><button onClick={() => setEditing(!editing)}>{editing ? t.cancel : t.editChinese}</button></div>
            {editing ? (
              <form onSubmit={(event) => { event.preventDefault(); void onSaveTranslation(skill.id, nameZh, summaryZh).then(() => setEditing(false)) }}>
                <input value={nameZh} onChange={(event) => setNameZh(event.target.value)} maxLength={80} />
                <textarea value={summaryZh} onChange={(event) => setSummaryZh(event.target.value)} maxLength={400} rows={4} />
                <button className="primary-button" disabled={workingKey === `translate:${skill.id}`}><Check size={15} />{t.save}</button>
              </form>
            ) : <div className="translation-card"><strong>{skill.nameZh}</strong><p>{skill.summaryZh}</p><small>{t.translationModes[skill.translationMode]}</small></div>}
          </details>

          <details className="drawer-section secondary-details">
            <summary>{t.toolsTitle}</summary>
            <div className="drawer-section-title"><strong>{t.toolsTitle}</strong><span>{t.connections} · {skill.connections.filter((item) => ['link', 'copy'].includes(item.mode)).length}</span></div>
            <div className="connection-list">
              {tools.map((tool) => {
                const connection = skill.connections.find((item) => item.toolId === tool.id)
                const connected = connection?.mode === 'link' || connection?.mode === 'copy'
                const independent = connection?.mode === 'conflict'
                const broken = connection?.mode === 'broken'
                const protectedEntry = independent || broken
                const status = broken ? t.brokenLink : independent ? t.unmanaged : connected ? t.connected : tool.detected ? t.notLinked : t.notDetected
                const action = broken ? t.needsReview : independent ? t.preserveEntry : connected ? t.disconnect : t.connect
                const busy = workingKey === `${skill.id}:${tool.id}`
                return (
                  <div key={tool.id}><span className="tool-mini">{tool.name.slice(0, 2).toUpperCase()}</span><span><strong>{tool.name}</strong><small>{status}</small></span><button className={connected ? 'disconnect' : ''} disabled={busy || protectedEntry || !isDesktopRuntime()} onClick={() => void onConnection(skill.id, tool.id, connected)}>{busy ? <LoaderCircle className="spin" size={14} /> : protectedEntry ? <ShieldCheck size={14} /> : connected ? <Unlink size={14} /> : <Link2 size={14} />}{action}</button></div>
                )
              })}
            </div>
            <p className="preview-note">{t.connectionNote}</p>
          </details>

          <details className="drawer-section secondary-details">
            <summary>{t.technicalDetails}</summary>
            <div className="drawer-section-title"><strong>{t.content}</strong><span>{skill.fileCount} {t.files}</span></div>
            <p className="preview-note">{t.score}: {skill.score} · {t.ready} ≠ {locale === 'zh' ? '实际调用已验证' : 'runtime verified'}</p>
            {contentError && <p className="inline-error">{contentError}</p>}
            {content ? <pre className="skill-source">{content.markdown}</pre> : !isDesktopRuntime() ? <p className="preview-note">{t.browserPreview}</p> : !contentError ? <div className="source-placeholder"><LoaderCircle className="spin" size={18} /></div> : null}
          </details>
        </div>
      </aside>
    </div>
  )
}
