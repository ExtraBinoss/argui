/** @jsxImportSource @argui/react */
import { memo, useCallback, useEffect, useMemo, useState, type ReactElement } from 'react'
import { VirtualList as ReactVirtualList, useThemeSnapshot } from '@argui/react'
import { Button as ReactButton, InputField as ReactInputField, accentOverrides, type Accent, type Palette } from '@argui/widgets/react'
import { ReactAnimationLab } from './animation-lab'
import { ReactOverlayPage } from './overlay-page'
import { ReactPopoverPage } from './popover-page'
import { ReactDamageControl } from './damage-control'
import { ReactAccessibilityPage } from './accessibility-page'
import { ReactInputsPage } from './input-page'
import { ReactWgslLab } from './wgsl-lab'
import { pages, filteredNavigation, navigationKey, navigationVersion, type Page } from '../gallery-pages'
import { ReactI18nPage } from './i18n-page'
import { ReactServicesPage } from './services-page'
import { ReactThemingPage } from './theming-page'
import { ReactDialogPage } from './dialog-page'
import type { ApplicationServices, ThemeRuntime } from '@argui/host'
import type { GalleryTokens } from '../theme'
import { isWidgetPage } from '../widget-pages'
import { WidgetPage } from './widget-pages'
import { ReactButtonPage } from './button-page'
import { ReactSelectPage, choices } from './select-page'
import { ReactMediaPage } from './media-page'
import { TypographyPage } from './typography-page'
import { DatePickerPage } from './date-picker-page'
import { DataTablePage } from './data-table-page'

const accents: readonly Accent[] = ['blue', 'violet', 'emerald']
const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Renders the same native gallery shell and pages through the React adapter. */
export function ReactGallery(props: { services: ApplicationServices; runtime: ThemeRuntime<GalleryTokens> }): ReactElement {
  const [page, setPage] = useState<Page>('Button')
  const [visited, setVisited] = useState<ReadonlySet<Page>>(() => new Set<Page>(['Button']))
  const snapshot = useThemeSnapshot(props.runtime)
  const [menuOpen, setMenuOpen] = useState(true)
  const [search, setSearch] = useState('')
  const [clicks, setClicks] = useState(0)
  const [lastUsed, setLastUsed] = useState('None')
  const [starActive, setStarActive] = useState(false)
  const [choice, setChoice] = useState<string>(choices[0])
  const selectPage = useCallback((item: Page) => {
    setVisited((previous) => new Set(previous).add(item))
    setPage(item)
  }, [])
  useEffect(() => props.services.onEvent((event) => {
    if (event.type === 'systemScheme') {
      props.runtime.update({ systemScheme: event.scheme })
      return
    }
    if ((event.type === 'menu' && event.id === 'open-services')
      || (event.type === 'shortcut' && event.id === 'wake' && event.state === 'pressed')) {
      selectPage('Services')
    }
  }), [props.services, props.runtime, selectPage])
  const theme = snapshot.values
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
      <ReactButton id="theme-toggle" label={snapshot.resolvedVariant === 'light' ? 'Dark theme' : 'Light theme'}
        theme={theme} kind="secondary" onClick={() => props.runtime.update({ variant: snapshot.resolvedVariant === 'light' ? 'dark' : 'light' })} />
      <ReactButton id="theme-system" label="System theme" theme={theme} kind="quiet"
        selected={snapshot.variant === 'system'} onClick={() => props.runtime.update({ variant: 'system' })} />
      {accents.map((option) => <ReactButton key={option} id={`accent-${option}`} label={option}
        theme={theme} kind="quiet" selected={theme.accent === accentOverrides[option].accent}
        onClick={() => props.runtime.update({ overrides: accentOverrides[option] })} />)}
    </row>
  )
  const navigation = menuOpen ? (
    <focusScope role="navigation" accessible_name="Gallery pages" focusable={false}
      width={mobile ? 'fill' : 220} height={mobile ? undefined : 'fill'}>
      <column width={mobile ? 'fill' : 220} height={mobile ? undefined : 'fill'} min_height={0}
        gap={8} padding={8}>
        <ReactInputField id="gallery-search" label="Search gallery" search theme={theme}
          value={search} placeholder="Search gallery" onChange={setSearch} />
        <ReactVirtualList id="gallery-navigation" count={navigationItems.length} estimate={mobile ? 112 : 47}
          itemKey={(index) => navigationKey(navigationItems[index]!)} dataVersion={navigationVersion(search)}
          variable={true} axis={mobile ? 'horizontal' : 'vertical'} overscan={3}
          width="fill" height={mobile ? 56 : 'fill'}
          scrollbarWidth={3} scrollbarColor={snapshot.resolvedVariant === 'dark' ? '#ffffff66' : '#0000003d'}
          shadow={{ color: theme.background, intensity: 1,
            width: 36, left: true, right: true, top: true, bottom: true }}
          renderItem={(index) => {
            const item = navigationItems[index]!
            if (item.kind === 'heading') return <column height={mobile ? 56 : 30} padding={7}
              justify_content="center"><text text={item.label} color={theme.muted} font_size={11} /></column>
            return <ReactButton id={navigationKey(item)} label={item.page} theme={theme}
              kind="ghost" selected={page === item.page}
              current={page === item.page ? 'page' : undefined} onClick={() => selectPage(item.page)} />
          }} />
        {navigationItems.length === 0 ? <text text="No pages found" color={theme.muted} font_size={12} /> : null}
      </column>
    </focusScope>
  ) : null
  // Preserve visited page state without mounting every unseen destination.
  const panes = pages.map((item) => (
    <column key={`scroll-${item}`} nativeKey={`scroll-${item}`} visible={page === item}
      width="fill" grow={1} min_width={0} min_height={0} scroll_y={true}>
      {visited.has(item) ? <ReactPageContent item={item} theme={theme} tokens={theme} runtime={props.runtime}
        services={props.services}
        selected={page === item}
        clicks={item === 'Button' ? clicks : undefined}
        lastUsed={item === 'Button' ? lastUsed : undefined}
        starActive={item === 'Button' ? starActive : undefined}
        activate={activate}
        choice={item === 'Select' ? choice : undefined}
        onChoiceChange={setChoice}
        active={item === 'Damage Control' || item === 'WGSL Lab' ? page === item : undefined} /> : null}
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
  tokens: GalleryTokens
  runtime: ThemeRuntime<GalleryTokens>
  services: ApplicationServices
  clicks?: number
  lastUsed?: string
  starActive?: boolean
  activate: (label: string) => void
  choice?: string
  onChoiceChange: (value: string) => void
  active?: boolean
  selected: boolean
}

/** Keeps retained page content stable while the shell changes native visibility. */
const ReactPageContent = memo(function ReactPageContent(props: ReactPageContentProps): ReactElement {
  const { item, theme } = props
  return (
    <column width="fill" min_width={mobile ? 0 : 260} shrink={1} gap={16} padding={mobile ? 4 : 18}>
      <text text={item} color={theme.foreground} font_size={26} />
      {item === 'Input' ? <ReactInputsPage theme={theme} /> : null}
      {item === 'Internationalization' && props.selected ? <ReactI18nPage theme={theme} /> : null}
      {item === 'Button' ? <ReactButtonPage theme={theme} clicks={props.clicks ?? 0}
        lastUsed={props.lastUsed ?? 'None'} starActive={props.starActive ?? false} activate={props.activate} /> : null}
      {item === 'Select' ? <ReactSelectPage theme={theme} value={props.choice ?? choices[0]}
        onChange={props.onChoiceChange} /> : null}
      {item === 'Popover' ? <ReactPopoverPage theme={theme} /> : null}
      {item === 'Dialog' ? <ReactDialogPage theme={theme} /> : null}
      {isWidgetPage(item) ? <WidgetPage name={item} theme={theme} /> : null}
      {item === 'Animation Lab' ? <ReactAnimationLab theme={theme} /> : null}
      {item === 'Media' ? <ReactMediaPage theme={theme} /> : null}
      {item === 'Services' ? <ReactServicesPage services={props.services} theme={theme} /> : null}
      {item === 'Theming' ? <ReactThemingPage theme={theme} tokens={props.tokens} runtime={props.runtime} /> : null}
      {item === 'Typography' ? <TypographyPage theme={theme} /> : null}
      {item === 'Date Picker' ? <DatePickerPage theme={theme} /> : null}
      {item === 'Data Table' ? <DataTablePage theme={theme} /> : null}
      {item === 'Accessibility' ? <ReactAccessibilityPage theme={theme} /> : null}
      {item === 'Overlay' ? <ReactOverlayPage theme={theme} /> : null}
      {item === 'Damage Control' ? <ReactDamageControl theme={theme} active={props.active ?? false} /> : null}
      {item === 'WGSL Lab' ? <ReactWgslLab theme={theme} active={props.active ?? false} /> : null}
    </column>
  )
}) as (props: ReactPageContentProps) => ReactElement
