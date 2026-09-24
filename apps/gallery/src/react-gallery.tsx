/** @jsxImportSource @argui/react */
import { memo, useCallback, useMemo, useState, type ReactElement } from 'react'
import { VirtualList as ReactVirtualList } from '@argui/react'
import { ReactButton, ReactSelect } from './react-controls'
import { ReactAnimationLab } from './react-animation-lab'
import { ReactOverlayPage, ReactPopoverPage } from './react-overlay'
import { ReactDamageControl } from './react-damage-control'
import { ReactInputField } from './react-input-field'
import { ReactAccessibilityPage } from './react-accessibility-page'
import { ReactInputsPage } from './react-inputs'
import { ReactWgslLab } from './react-wgsl-lab'
import { pages, filteredNavigation, navigationKey, navigationVersion, type Page } from './gallery-pages'
import { mediaAssets } from './assets.generated'
import { palette, type Accent, type Palette, type ThemeMode } from './theme'

const accents: readonly Accent[] = ['blue', 'violet', 'emerald']
const choices = ['Vulkan', 'DirectX 12', 'Metal', 'WebGPU'] as const
const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Renders the same native gallery shell and pages through the React adapter. */
export function ReactGallery(): ReactElement {
  const [page, setPage] = useState<Page>('Button')
  const [mode, setMode] = useState<ThemeMode>('light')
  const [accent, setAccent] = useState<Accent>('blue')
  const [menuOpen, setMenuOpen] = useState(true)
  const [search, setSearch] = useState('')
  const [clicks, setClicks] = useState(0)
  const [lastUsed, setLastUsed] = useState('None')
  const [starActive, setStarActive] = useState(false)
  const [choice, setChoice] = useState<string>(choices[0])
  const theme = useMemo(() => palette(mode, accent), [mode, accent])
  const navigationItems = useMemo(() => filteredNavigation(search), [search])
  const activate = useCallback((label: string) => {
    setClicks((value) => value + 1)
    setLastUsed(label)
    setStarActive((value) => label === 'Star' ? !value : false)
  }, [])
  const menuButton = <ReactButton id="menu" label={menuOpen ? 'Hide navigation' : 'Show navigation'}
    theme={theme} kind="ghost" onClick={() => setMenuOpen((value) => !value)} />
  const themeControls = (
    <row wrap={true} gap={8}>
      <ReactButton id="theme-toggle" label={mode === 'light' ? 'Dark theme' : 'Light theme'}
        theme={theme} kind="secondary" onClick={() => setMode(mode === 'light' ? 'dark' : 'light')} />
      {accents.map((option) => <ReactButton key={option} id={`accent-${option}`} label={option}
        theme={theme} kind="quiet" selected={accent === option} onClick={() => setAccent(option)} />)}
    </row>
  )
  const navigation = menuOpen ? (
    <focusScope role="navigation" accessible_name="Gallery pages" focusable={false}
      width={mobile ? 'fill' : 220} height={mobile ? undefined : 'fill'}>
      <column width={mobile ? 'fill' : 220} height={mobile ? undefined : 'fill'} min_height={0}
        gap={8} padding={8}>
        <ReactInputField id="gallery-search" label="Search components" search theme={theme}
          value={search} placeholder="Search components" onChange={setSearch} />
        <ReactVirtualList id="gallery-navigation" count={navigationItems.length} estimate={mobile ? 112 : 47}
          itemKey={(index) => navigationKey(navigationItems[index]!)} dataVersion={navigationVersion(search)}
          variable={true} axis={mobile ? 'horizontal' : 'vertical'} overscan={3}
          width="fill" height={mobile ? 56 : 'fill'}
          scrollbarWidth={3} scrollbarColor={mode === 'dark' ? '#ffffff66' : '#0000003d'}
          shadow={{ color: theme.background, intensity: 1,
            width: 36, left: true, right: true, top: true, bottom: true }}
          renderItem={(index) => {
            const item = navigationItems[index]!
            if (item.kind === 'heading') return <column height={mobile ? 56 : 30} padding={7}
              justify_content="center"><text text={item.label} color={theme.muted} font_size={11} /></column>
            return <ReactButton id={navigationKey(item)} label={item.page} theme={theme}
              kind="ghost" selected={page === item.page}
              current={page === item.page ? 'page' : undefined} onClick={() => setPage(item.page)} />
          }} />
        {navigationItems.length === 0 ? <text text="No components found" color={theme.muted} font_size={12} /> : null}
      </column>
    </focusScope>
  ) : null
  // Preserve page subtrees across tab changes; invisible native branches do not animate.
  const panes = pages.map((item) => (
    <column key={`scroll-${item}`} nativeKey={`scroll-${item}`} visible={page === item}
      width="fill" grow={1} min_width={0} min_height={0} scroll_y={true}>
      <ReactPageContent item={item} theme={theme}
        clicks={item === 'Button' ? clicks : undefined}
        lastUsed={item === 'Button' ? lastUsed : undefined}
        starActive={item === 'Button' ? starActive : undefined}
        activate={activate}
        choice={item === 'Select' ? choice : undefined}
        onChoiceChange={setChoice}
        active={item === 'Damage Control' || item === 'WGSL Lab' ? page === item : undefined} />
    </column>
  ))

  return (
    <column width="fill" height="fill" background={theme.background}>
      {mobile ? (
        <column width="fill" gap={9} padding={12} background={theme.surface}>
          <text text="ARGUI / Native Gallery" color={theme.foreground} font_size={18} />
          <row width="fill" wrap={true} gap={8}>{menuButton}{themeControls}</row>
        </column>
      ) : (
        <row width="fill" wrap={true} gap={12} padding={12} background={theme.surface}>
          {menuButton}
          <text text="ARGUI / Native Gallery" color={theme.foreground} font_size={18} />
          {themeControls}
        </row>
      )}
      {mobile ? <column width="fill" height="fill" shrink={1} min_height={0} gap={16}>
        {navigation}
        <column width="fill" grow={1} shrink={1} min_height={0}
          padding_left={12} padding_right={12} padding_bottom={12}>{panes}</column>
      </column> : <row width="fill" height="fill" shrink={1} min_height={0} gap={16} padding={16}>
        {navigation}
        {panes}
      </row>}
    </column>
  )
}

interface ReactPageContentProps {
  item: Page
  theme: Palette
  clicks?: number
  lastUsed?: string
  starActive?: boolean
  activate: (label: string) => void
  choice?: string
  onChoiceChange: (value: string) => void
  active?: boolean
}

/** Keeps retained page content stable while the shell changes native visibility. */
const ReactPageContent = memo(function ReactPageContent(props: ReactPageContentProps): ReactElement {
  const { item, theme } = props
  return (
    <column width="fill" min_width={mobile ? 0 : 260} shrink={1} gap={16} padding={mobile ? 4 : 18}>
      <text text={item} color={theme.foreground} font_size={26} />
      {item === 'Input' ? <ReactInputsPage theme={theme} /> : null}
      {item === 'Button' ? <ReactButtonPage theme={theme} clicks={props.clicks ?? 0}
        lastUsed={props.lastUsed ?? 'None'} starActive={props.starActive ?? false} activate={props.activate} /> : null}
      {item === 'Select' ? <ReactSelectPage theme={theme} value={props.choice ?? choices[0]}
        onChange={props.onChoiceChange} /> : null}
      {item === 'Popover' ? <ReactPopoverPage theme={theme} /> : null}
      {item === 'Animation Lab' ? <ReactAnimationLab theme={theme} /> : null}
      {item === 'Media' ? <ReactMediaPage theme={theme} /> : null}
      {item === 'Accessibility' ? <ReactAccessibilityPage theme={theme} /> : null}
      {item === 'Overlay' ? <ReactOverlayPage theme={theme} /> : null}
      {item === 'Damage Control' ? <ReactDamageControl theme={theme} active={props.active ?? false} /> : null}
      {item === 'WGSL Lab' ? <ReactWgslLab theme={theme} active={props.active ?? false} /> : null}
    </column>
  )
}) as (props: ReactPageContentProps) => ReactElement

/** Keeps button activation state in the shell while demonstrating every variant. */
function ReactButtonPage(props: { theme: Palette; clicks: number; lastUsed: string; starActive: boolean; activate: (label: string) => void }): ReactElement {
  return (
    <column width="fill" gap={18}>
      <text width="fill" text="Actions, variants and interactive states."
        color={props.theme.muted} font_size={14} />
      <text text="Variants" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <ReactButton id="button-primary" label="Primary" theme={props.theme} kind="primary" onClick={() => props.activate('Primary')} />
        <ReactButton id="button-secondary" label="Secondary" theme={props.theme} kind="secondary" onClick={() => props.activate('Secondary')} />
        <ReactButton id="button-outline" label="Outline" theme={props.theme} kind="outline" onClick={() => props.activate('Outline')} />
        <ReactButton id="button-ghost" label="Ghost" theme={props.theme} kind="ghost" onClick={() => props.activate('Ghost')} />
        <ReactButton id="button-destructive" label="Delete" theme={props.theme} kind="destructive" onClick={() => props.activate('Delete')} />
      </row>
      <text text="States and icons" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <ReactButton id="button-disabled" label="Disabled" theme={props.theme} kind="outline" disabled onClick={() => props.activate('Disabled')} />
        <ReactButton id="button-busy" label="Loading" theme={props.theme} kind="primary" busy onClick={() => props.activate('Loading')} />
        <ReactButton id="button-star" label="Star" theme={props.theme} kind="outline" selected={props.starActive}
          icon={mediaAssets['tabler/star.svg']} activeIcon={mediaAssets['tabler/star-filled.svg']}
          iconOnly onClick={() => props.activate('Star')} />
      </row>
      <text text={`Clicks: ${props.clicks} · Last used: ${props.lastUsed}`}
        color={props.theme.muted} font_size={13} />
    </column>
  )
}

/** Shows controlled native selection with the same options as Solid. */
function ReactSelectPage(props: { theme: Palette; value: string; onChange: (value: string) => void }): ReactElement {
  return (
    <column gap={14}>
      <text width="fill" text="Choose the backend for your next render."
        color={props.theme.muted} font_size={14} />
      <ReactSelect id="topic-select" label="Choose a backend" options={choices} value={props.value}
        theme={props.theme} onChange={props.onChange} />
      <text text={`Selected ${props.value}`} color={props.theme.muted} font_size={13} />
    </column>
  )
}

/** Shows native Tabler SVGs and embedded raster illustrations like the Solid page. */
function ReactMediaPage(props: { theme: Palette }): ReactElement {
  return (
    <column gap={16}>
      <text width="fill" text="Imported Tabler SVGs and raster images."
        color={props.theme.muted} font_size={14} />
      <row width="fill" wrap={true} gap={20}>
        <column width={mobile ? 'fill' : 240} gap={8}>
          <text text="Tabler SVG icons" color={props.theme.foreground} font_size={16} />
          <row gap={12}>
            <svg source={mediaAssets['tabler/heart.svg']} color={props.theme.accent} width={36} height={36} />
            <svg source={mediaAssets['tabler/photo.svg']} color={props.theme.accent} width={36} height={36} />
            <svg source={mediaAssets['tabler/player-play.svg']} color={props.theme.accent} width={36} height={36} />
          </row>
        </column>
        <column width={mobile ? 'fill' : 240} height={mobile ? 260 : 200} gap={8}>
          <text text="Multicolor SVG pre-rendered as PNG" color={props.theme.foreground} font_size={16} />
          <image source={mediaAssets['illustration/orbit.png']} alt="Colorful orbit illustration" width="fill" height={160} fit="contain" />
        </column>
        <column width={mobile ? 'fill' : 240} height={mobile ? 260 : 200} gap={8}>
          <text text="Imported JPEG image" color={props.theme.foreground} font_size={16} />
          <image source={mediaAssets['photo/saturn.jpg']} alt="Saturn with its rings" width="fill" height={160} fit="cover" />
        </column>
      </row>
    </column>
  )
}
