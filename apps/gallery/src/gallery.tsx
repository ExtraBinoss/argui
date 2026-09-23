import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, Select } from './controls'
import { AnimationLab } from './animation-lab'
import { mediaAssets } from './assets.generated'
import { palette, type Accent, type Palette, type ThemeMode } from './theme'

type Page = 'Button' | 'Select' | 'Animation Lab' | 'Media'
const pages: readonly Page[] = ['Button', 'Select', 'Animation Lab', 'Media']
const accents: readonly Accent[] = ['blue', 'violet', 'emerald']
const choices = ['Vulkan', 'DirectX 12', 'Metal', 'WebGPU'] as const
const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** The navigable native gallery shell with shared theme and page state. */
export function Gallery(): JSX.Element {
  const [page, setPage] = createSignal<Page>('Button')
  const [mode, setMode] = createSignal<ThemeMode>('light')
  const [accent, setAccent] = createSignal<Accent>('blue')
  const [menuOpen, setMenuOpen] = createSignal(true)
  const [clicks, setClicks] = createSignal(0)
  const [lastUsed, setLastUsed] = createSignal('None')
  const [starActive, setStarActive] = createSignal(false)
  const [choice, setChoice] = createSignal<string>(choices[0])
  const theme = () => palette(mode(), accent())
  const activate = (label: string) => {
    setClicks((value) => value + 1)
    setLastUsed(label)
    setStarActive(label === 'Star' ? !starActive() : false)
  }
  const menuButton = () => <Button id="menu" label={menuOpen() ? 'Hide navigation' : 'Show navigation'}
    theme={theme()} kind="quiet" onClick={() => setMenuOpen(!menuOpen())} />
  const themeControls = () => (
    <row wrap={true} gap={8}>
      <Button id="theme-toggle" label={mode() === 'light' ? 'Dark theme' : 'Light theme'}
        theme={theme()} kind="secondary" onClick={() => setMode(mode() === 'light' ? 'dark' : 'light')} />
      {accents.map((option) => <Button id={`accent-${option}`} label={option} theme={theme()}
        kind="quiet" selected={accent() === option} onClick={() => setAccent(option)} />)}
    </row>
  )
  const navigation = () => menuOpen() ? (
    <focusScope role="navigation" accessible_name="Gallery pages" focus_on_tab_navigation={false} width={mobile ? 'fill' : 150}>
      <column width={mobile ? 'fill' : 150} gap={7} padding={mobile ? 0 : 10}>
        <text text="EXPLORE" color={theme().muted} font_size={11} />
        {mobile ? <row width="fill" wrap={true} gap={7}>
          {pages.map((item) => <Button id={`page-${item.replaceAll(' ', '-').toLowerCase()}`}
            label={item} theme={theme()}
            kind={page() === item ? 'secondary' : 'quiet'} selected={page() === item}
            current={page() === item ? 'page' : undefined} onClick={() => setPage(item)} />)}
        </row> : <column gap={7}>
          {pages.map((item) => <Button id={`page-${item.replaceAll(' ', '-').toLowerCase()}`}
            label={item} theme={theme()}
            kind={page() === item ? 'secondary' : 'quiet'} selected={page() === item}
            current={page() === item ? 'page' : undefined} onClick={() => setPage(item)} />)}
        </column>}
      </column>
    </focusScope>
  ) : null
  const content = () => (
    <column width="fill" min_width={mobile ? 0 : 260} shrink={1} gap={16} padding={mobile ? 4 : 18}>
      <text text={page()} color={theme().foreground} font_size={26} />
      {page() === 'Button' ? <ButtonPage theme={theme()} clicks={clicks()}
        lastUsed={lastUsed()} starActive={starActive()} activate={activate} /> : null}
      {page() === 'Select' ? <SelectPage theme={theme()} value={choice()} onChange={setChoice} /> : null}
      {page() === 'Animation Lab' ? <AnimationLab theme={theme()} /> : null}
      {page() === 'Media' ? <MediaPage theme={theme()} /> : null}
    </column>
  )
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
        <column width="fill" height="fill" shrink={1} min_height={0} gap={16} padding={12}>
          {navigation()}
          <column key={`scroll-${page()}`} width="fill" grow={1} min_height={0} scroll_y={true}>
            {content()}
          </column>
        </column>
      ) : (
        <row width="fill" height="fill" shrink={1} min_height={0} gap={16} padding={16}>
          {navigation()}
          <column key={`scroll-${page()}`} grow={1} min_width={0} min_height={0} scroll_y={true}>
            {content()}
          </column>
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
          <focusScope role="image" accessible_name="Colorful orbit illustration" focus_on_tab_navigation={false} width="fill" height={160}>
            <image source={mediaAssets['illustration/orbit.png']} width="fill" height={160} fit="contain" />
          </focusScope>
        </column>
        <column width={mobile ? 'fill' : 240} height={mobile ? 260 : 200} gap={8}>
          <text text="Imported JPEG image" color={props.theme.foreground} font_size={16} />
          <focusScope role="image" accessible_name="Saturn with its rings" focus_on_tab_navigation={false} width="fill" height={160}>
            <image source={mediaAssets['photo/saturn.jpg']} width="fill" height={160} fit="cover" />
          </focusScope>
        </column>
      </row>
    </column>
  )
}
