import { createMemo, createSignal, onCleanup, VirtualList } from '@argui/solid'
import { batch } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, InputField, Select, palette, type Accent, type Palette, type ThemeMode } from '@argui/widgets/solid'
import { AnimationLab } from './animation-lab'
import { OverlayPage, PopoverPage } from './overlay'
import { DamageControl } from './damage-control'
import { AccessibilityPage } from './accessibility-page'
import { InputsPage } from './inputs'
import { WgslLab } from './wgsl-lab'
import { pages, filteredNavigation, navigationKey, navigationVersion, type Page } from '../gallery-pages'
import { mediaAssets } from '../assets.generated'
import { I18nPage } from './i18n-page'
import { ServicesPage } from './services-page'
import { ThemingPage } from './theming-page'
import { DialogPage } from './dialog-page'
import type { ApplicationServices } from '@argui/host'

const accents: readonly Accent[] = ['blue', 'violet', 'emerald']
const choices = ['Vulkan', 'DirectX 12', 'Metal', 'WebGPU'] as const
const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** The navigable native gallery shell with shared theme and page state. */
export function Gallery(props: { services: ApplicationServices }): JSX.Element {
  const [page, setPage] = createSignal<Page>('Button')
  const [visited, setVisited] = createSignal<ReadonlySet<Page>>(new Set<Page>(['Button']))
  const [mode, setMode] = createSignal<ThemeMode>('light')
  const [accent, setAccent] = createSignal<Accent>('blue')
  const [menuOpen, setMenuOpen] = createSignal(true)
  const [search, setSearch] = createSignal('')
  const [clicks, setClicks] = createSignal(0)
  const [lastUsed, setLastUsed] = createSignal('None')
  const [starActive, setStarActive] = createSignal(false)
  const [choice, setChoice] = createSignal<string>(choices[0])
  const selectPage = (item: Page) => batch(() => {
    setVisited((previous) => new Set(previous).add(item))
    setPage(item)
  })
  onCleanup(props.services.onEvent((event) => {
    if ((event.type === 'menu' && event.id === 'open-services')
      || (event.type === 'shortcut' && event.id === 'wake' && event.state === 'pressed')) {
      selectPage('Services')
    }
  }))
  const theme = () => palette(mode(), accent())
  const activate = (label: string) => {
    setClicks((value) => value + 1)
    setLastUsed(label)
    setStarActive(label === 'Star' ? !starActive() : false)
  }
  const menuButton = () => <Button id="menu" label={menuOpen() ? 'Hide navigation' : 'Show navigation'}
    theme={theme()} kind="ghost" onClick={() => setMenuOpen(!menuOpen())} />
  const themeControls = () => (
    <row wrap={true} gap={8}>
      <Button id="theme-toggle" label={mode() === 'light' ? 'Dark theme' : 'Light theme'}
        theme={theme()} kind="secondary" onClick={() => setMode(mode() === 'light' ? 'dark' : 'light')} />
      {accents.map((option) => <Button id={`accent-${option}`} label={option} theme={theme()}
        kind="quiet" selected={accent() === option} onClick={() => setAccent(option)} />)}
    </row>
  )
  const navigation = () => {
    if (!menuOpen()) return null
    const items = createMemo(() => filteredNavigation(search()))
    return <focusScope role="navigation" accessible_name="Gallery pages" focusable={false}
      width={mobile ? 'fill' : 220} height={mobile ? undefined : 'fill'}>
      <column width={mobile ? 'fill' : 220} height={mobile ? undefined : 'fill'} min_height={0}
        gap={8} padding={8}>
        <InputField id="gallery-search" label="Search gallery" search theme={theme()}
          value={search()} placeholder="Search gallery" onChange={setSearch} />
        <VirtualList id="gallery-navigation" count={items().length} estimate={mobile ? 112 : 47}
          itemKey={(index) => navigationKey(items()[index]!)} dataVersion={navigationVersion(search())}
          variable={true} axis={mobile ? 'horizontal' : 'vertical'} overscan={3}
          width="fill" height={mobile ? 56 : 'fill'}
          scrollbarWidth={3} scrollbarColor={mode() === 'dark' ? '#ffffff66' : '#0000003d'}
          shadow={{ color: theme().background, intensity: 1,
            width: 36, left: true, right: true, top: true, bottom: true }}
          renderItem={(index) => {
            const item = items()[index]!
            if (item.kind === 'heading') return <column height={mobile ? 56 : 30} padding={7}
              justify_content="center"><text text={item.label} color={theme().muted} font_size={11} /></column>
            return <Button id={navigationKey(item)} label={item.page} theme={theme()}
              kind="ghost" selected={page() === item.page}
              current={page() === item.page ? 'page' : undefined} onClick={() => selectPage(item.page)} />
          }} />
        {items().length === 0 ? <text text="No pages found" color={theme().muted} font_size={12} /> : null}
      </column>
    </focusScope>
  }
  // Keep visited destinations mounted so page state survives navigation.
  // Unvisited pages add no hidden layout or paint work.
  const content = (item: Page) => (
    <column width="fill" min_width={mobile ? 0 : 260} shrink={1} gap={16} padding={mobile ? 4 : 18}>
      <text text={item} color={theme().foreground} font_size={26} />
      {item === 'Input' ? <InputsPage theme={theme()} /> : null}
      {item === 'Internationalization' && page() === item ? <I18nPage theme={theme()} /> : null}
      {item === 'Button' ? <ButtonPage theme={theme()} clicks={clicks()}
        lastUsed={lastUsed()} starActive={starActive()} activate={activate} /> : null}
      {item === 'Select' ? <SelectPage theme={theme()} value={choice()} onChange={setChoice} /> : null}
      {item === 'Popover' ? <PopoverPage theme={theme()} /> : null}
      {item === 'Dialog' ? <DialogPage theme={theme()} /> : null}
      {item === 'Animation Lab' ? <AnimationLab theme={theme()} /> : null}
      {item === 'Media' ? <MediaPage theme={theme()} /> : null}
      {item === 'Services' ? <ServicesPage services={props.services} theme={theme()} /> : null}
      {item === 'Theming' ? <ThemingPage theme={theme()} /> : null}
      {item === 'Accessibility' ? <AccessibilityPage theme={theme()} /> : null}
      {item === 'Overlay' ? <OverlayPage theme={theme()} /> : null}
      {item === 'Damage Control' ? <DamageControl theme={theme()} active={page() === item} /> : null}
      {item === 'WGSL Lab' ? <WgslLab theme={theme()} active={page() === item} /> : null}
    </column>
  )
  const panes = () => pages.map((item) => (
    <column key={`scroll-${item}`} visible={page() === item}
      width="fill" grow={1} min_width={0} min_height={0} scroll_y={true}>
      {visited().has(item) ? content(item) : null}
    </column>
  ))
  return (
    <column width="fill" height="fill" background={theme().background}>
      {mobile ? (
        <column width="fill" gap={9} padding={12} background={theme().surface}>
          <text text="ARGUI / Native Gallery" color={theme().foreground} font_size={18} />
          <row width="fill" wrap={true} gap={8}>{menuButton()}{themeControls()}</row>
        </column>
      ) : (
        <row width="fill" wrap={true} gap={12} padding={12} background={theme().surface}>
          {menuButton()}
          <text text="ARGUI / Native Gallery" color={theme().foreground} font_size={18} />
          {themeControls()}
        </row>
      )}
      {mobile ? (
        <column width="fill" height="fill" shrink={1} min_height={0} gap={16}>
          {navigation()}
          <column width="fill" grow={1} shrink={1} min_height={0}
            padding_left={12} padding_right={12} padding_bottom={12}>{panes()}</column>
        </column>
      ) : (
        <row width="fill" height="fill" shrink={1} min_height={0} gap={16} padding={16}>
          {navigation()}
          {panes()}
        </row>
      )}
    </column>
  )
}

/** Shows the native button variants, blocked states, spinner and star toggle. */
function ButtonPage(props: { theme: Palette; clicks: number; lastUsed: string; starActive: boolean; activate: (label: string) => void }): JSX.Element {
  return (
    <column width="fill" gap={18}>
      <text text="Actions, variants and interactive states." width="fill" color={props.theme.muted} font_size={14} />
      <text text="Variants" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <Button id="button-primary" label="Primary" theme={props.theme} kind="primary" onClick={() => props.activate('Primary')} />
        <Button id="button-secondary" label="Secondary" theme={props.theme} kind="secondary" onClick={() => props.activate('Secondary')} />
        <Button id="button-outline" label="Outline" theme={props.theme} kind="outline" onClick={() => props.activate('Outline')} />
        <Button id="button-ghost" label="Ghost" theme={props.theme} kind="ghost" onClick={() => props.activate('Ghost')} />
        <Button id="button-destructive" label="Delete" theme={props.theme} kind="destructive" onClick={() => props.activate('Delete')} />
      </row>
      <text text="States and icons" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <Button id="button-disabled" label="Disabled" theme={props.theme} kind="outline" disabled onClick={() => props.activate('Disabled')} />
        <Button id="button-busy" label="Loading" theme={props.theme} kind="primary" busy onClick={() => props.activate('Loading')} />
        <Button id="button-star" label="Star" theme={props.theme} kind="outline" selected={props.starActive}
          icon={mediaAssets['tabler/star.svg']} activeIcon={mediaAssets['tabler/star-filled.svg']}
          iconOnly onClick={() => props.activate('Star')} />
      </row>
      <text text={`Clicks: ${props.clicks} · Last used: ${props.lastUsed}`}
        color={props.theme.muted} font_size={13} />
    </column>
  )
}

/** Shows a controlled selector backed by the native popup and focus boundary. */
function SelectPage(props: { theme: Palette; value: string; onChange: (value: string) => void }): JSX.Element {
  return (
    <column gap={14}>
      <text text="Choose the backend for your next render." width="fill" color={props.theme.muted} font_size={14} />
      <Select id="topic-select" label="Choose a backend" options={choices} value={props.value} theme={props.theme} onChange={props.onChange} />
      <text text={`Selected ${props.value}`} color={props.theme.muted} font_size={13} />
    </column>
  )
}

/** Shows native Tabler SVGs and embedded raster illustrations in both adapters. */
function MediaPage(props: { theme: Palette }): JSX.Element {
  return (
    <column gap={16}>
      <text text="Imported Tabler SVGs and raster images." width="fill" color={props.theme.muted} font_size={14} />
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
