import { createMemo, createSignal, onCleanup, useThemeSnapshot, VirtualList } from '@argui/solid'
import { batch } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, InputField, accentOverrides, type Accent, type Palette } from '@argui/widgets/solid'
import { AnimationLab } from './animation-lab'
import { OverlayPage } from './overlay-page'
import { PopoverPage } from './popover-page'
import { DamageControl } from './damage-control'
import { AccessibilityPage } from './accessibility-page'
import { InputsPage } from './input-page'
import { WgslLab } from './wgsl-lab'
import { pages, filteredNavigation, navigationKey, navigationVersion, type Page } from '../gallery-pages'
import { I18nPage } from './i18n-page'
import { ServicesPage } from './services-page'
import { ThemingPage } from './theming-page'
import { DialogPage } from './dialog-page'
import type { ApplicationServices } from '@argui/host'
import type { ThemeRuntime } from '@argui/host'
import type { GalleryTokens } from '../theme'
import { isWidgetPage } from '../widget-pages'
import { WidgetPage } from './widget-pages'
import { ButtonPage } from './button-page'
import { SelectPage, choices } from './select-page'
import { MediaPage } from './media-page'
import { TypographyPage } from './typography-page'
import { DatePickerPage } from './date-picker-page'
import { DataTablePage } from './data-table-page'

const accents: readonly Accent[] = ['blue', 'violet', 'emerald']
const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** The navigable native gallery shell with shared theme and page state. */
export function Gallery(props: { services: ApplicationServices; runtime: ThemeRuntime<GalleryTokens> }): JSX.Element {
  const [page, setPage] = createSignal<Page>('Button')
  const [visited, setVisited] = createSignal<ReadonlySet<Page>>(new Set<Page>(['Button']))
  const snapshot = useThemeSnapshot(props.runtime)
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
    if (event.type === 'systemScheme') {
      props.runtime.update({ systemScheme: event.scheme })
      return
    }
    if ((event.type === 'menu' && event.id === 'open-services')
      || (event.type === 'shortcut' && event.id === 'wake' && event.state === 'pressed')) {
      selectPage('Services')
    }
  }))
  const theme = () => snapshot().values
  const activate = (label: string) => {
    setClicks((value) => value + 1)
    setLastUsed(label)
    setStarActive(label === 'Star' ? !starActive() : false)
  }
  const menuButton = () => <Button id="menu" label={menuOpen() ? 'Hide navigation' : 'Show navigation'}
    theme={theme()} kind="ghost" onClick={() => setMenuOpen(!menuOpen())} />
  const themeControls = () => (
    <row wrap={true} gap={8}>
      <Button id="theme-toggle" label={snapshot().resolvedVariant === 'light' ? 'Dark theme' : 'Light theme'}
        theme={theme()} kind="secondary" onClick={() => props.runtime.update({ variant: snapshot().resolvedVariant === 'light' ? 'dark' : 'light' })} />
      <Button id="theme-system" label="System theme" theme={theme()} kind="quiet"
        selected={snapshot().variant === 'system'} onClick={() => props.runtime.update({ variant: 'system' })} />
      {accents.map((option) => <Button id={`accent-${option}`} label={option} theme={theme()}
        kind="quiet" selected={theme().accent === accentOverrides[option].accent}
        onClick={() => props.runtime.update({ overrides: accentOverrides[option] })} />)}
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
          scrollbarWidth={3} scrollbarColor={snapshot().resolvedVariant === 'dark' ? '#ffffff66' : '#0000003d'}
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
      {isWidgetPage(item) ? <WidgetPage name={item} theme={theme()} /> : null}
      {item === 'Animation Lab' ? <AnimationLab theme={theme()} /> : null}
      {item === 'Media' ? <MediaPage theme={theme()} /> : null}
      {item === 'Services' ? <ServicesPage services={props.services} theme={theme()} /> : null}
      {item === 'Theming' ? <ThemingPage theme={theme()} tokens={theme()} runtime={props.runtime} /> : null}
      {item === 'Typography' ? <TypographyPage theme={theme()} /> : null}
      {item === 'Date Picker' ? <DatePickerPage theme={theme()} /> : null}
      {item === 'Data Table' ? <DataTablePage theme={theme()} /> : null}
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
