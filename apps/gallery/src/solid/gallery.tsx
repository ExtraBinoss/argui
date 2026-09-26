import { For, createSignal } from 'solid-js'
import type { ThemeRuntime } from '@argui/host'
import { ThemeScope, useTheme, useThemeSnapshot } from '@argui/solid'
import { Button, ButtonGroup, ButtonGroupSeparator, InputField, Popover, type WidgetTheme } from '@argui/widgets/solid'
import { ButtonPage } from './button-page'
import { ButtonGroupPage } from './button-group-page'
import { InputFieldPage } from './input-field-page'
import { SelectPage } from './select-page'
import { PopoverPage } from './popover-page'
import { VirtualListPage } from './virtual-list-page'
import { LayoutScenarios } from './layout-scenarios'
import { AnimationPage } from './animation-page'
import { mediaAssets } from '../../assets.generated'

const pages = [
  { id: 'button', label: 'Button', category: 'Widgets' },
  { id: 'button-group', label: 'ButtonGroup', category: 'Widgets' },
  { id: 'input-field', label: 'InputField', category: 'Widgets' },
  { id: 'select', label: 'Select', category: 'Widgets' },
  { id: 'popover', label: 'Popover', category: 'Widgets' },
  { id: 'virtual-list', label: 'VirtualList', category: 'Widgets' },
  { id: 'layouting', label: 'Layouting', category: 'Examples' },
  { id: 'animation', label: 'Animation', category: 'Examples' },
] as const

type PageId = typeof pages[number]['id']

/** Renders a searchable sidebar, one widget page, and a compact settings popover. */
export function Gallery(props: { runtime: ThemeRuntime<WidgetTheme> }) {
  const [page, setPage] = createSignal<PageId>('button')
  const [search, setSearch] = createSignal('')
  const [settingsOpen, setSettingsOpen] = createSignal(false)
  const theme = useTheme<WidgetTheme>()
  const snapshot = useThemeSnapshot<WidgetTheme>()
  const selectedTheme = () => snapshot().variant ?? 'system'
  const visiblePages = (category: 'Widgets' | 'Examples') => pages.filter((entry) => entry.category === category && entry.label.toLowerCase().includes(search().trim().toLowerCase()))
  const content = () => page() === 'button' ? <ButtonPage />
    : page() === 'button-group' ? <ButtonGroupPage />
    : page() === 'input-field' ? <InputFieldPage />
    : page() === 'select' ? <SelectPage />
    : page() === 'popover' ? <PopoverPage />
    : page() === 'virtual-list' ? <VirtualListPage />
    : page() === 'layouting' ? <column width="100%" gap={16}><text color={theme().text} fontSize={24}>Layouting</text><LayoutScenarios /></column>
    : <AnimationPage />

  return <row width="100%" height="100%" background={theme().surface}>
    <column id="gallery-sidebar" width={256} height="100%" shrink={0} padding={16} gap={16} background={theme().sidebar}>
      <container padding={{ start: 8 }}><text color={theme().text} fontSize={20}>Argui</text></container>
      <InputField id="gallery-search" accessibleName="Search gallery" type="search"
        placeholder="Search gallery" value={search()} onValueChange={setSearch}
        leading={<svg source={mediaAssets['tabler/search.svg']} width={16} height={16} color={theme().textMuted} />} />
      <ThemeScope<WidgetTheme> overrides={{ ghostHover: theme().sidebarAccent }}>
      <scrollView id="gallery-navigation" width="100%" height="100%" grow={1} scrollY={true}
        scrollbarSide="left" scrollbarWidth={3} scrollbarThumbColor={theme().border} scrollbarHoverColor={theme().textMuted}>
        <column width="100%" gap={12}>
          <For each={['Widgets', 'Examples'] as const}>{(category) => visiblePages(category).length > 0 && <column width="100%" gap={4}>
            <container padding={{ start: 8, bottom: 4 }}><text color={theme().textMuted} fontSize={12}>{category}</text></container>
            <For each={visiblePages(category)}>{(entry) => <Button
              id={`page-${entry.id}`}
              width="100%"
              contentAlign="start"
              variant="ghost"
              pressed={page() === entry.id}
              onClick={() => setPage(entry.id)}
            ><text color={page() === entry.id ? theme().sidebarPrimary : theme().sidebarForeground}>{entry.label}</text></Button>}</For>
          </column>}</For>
        </column>
      </scrollView>
      </ThemeScope>
    </column>
    <column width="100%" height="100%" grow={1} minWidth={0}>
      <row width="100%" padding={{ top: 12, right: 16, bottom: 4, left: 16 }} justifyContent="end">
        <Popover id="gallery-settings" trigger="Settings" placement="bottomEnd" contentWidth={280} blur={true}
          open={settingsOpen()} onOpenChange={setSettingsOpen}
          leading={<svg source={mediaAssets['gallery/settings.svg']} width={16} height={16} color={settingsOpen() ? theme().primary : theme().text} />}>
          <text color={theme().text} fontSize={14}>Appearance</text>
          <ButtonGroup accessibleName="Theme">
            <Button id="theme-light" pressed={selectedTheme() === 'light'} variant="ghost"
              onClick={() => props.runtime.update({ variant: 'light' })}>
              <row gap={6} alignItems="center">
                <svg source={mediaAssets['gallery/sun.svg']} width={16} height={16} color={selectedTheme() === 'light' ? theme().primary : theme().text} />
                <text color={selectedTheme() === 'light' ? theme().primary : theme().text}>Light</text>
              </row>
            </Button>
            <ButtonGroupSeparator />
            <Button id="theme-dark" pressed={selectedTheme() === 'dark'} variant="ghost"
              onClick={() => props.runtime.update({ variant: 'dark' })}>
              <row gap={6} alignItems="center">
                <svg source={mediaAssets['gallery/moon.svg']} width={16} height={16} color={selectedTheme() === 'dark' ? theme().primary : theme().text} />
                <text color={selectedTheme() === 'dark' ? theme().primary : theme().text}>Dark</text>
              </row>
            </Button>
            <ButtonGroupSeparator />
            <Button id="theme-system" pressed={selectedTheme() === 'system'} variant="ghost"
              onClick={() => props.runtime.update({ variant: 'system' })}>
              <row gap={6} alignItems="center">
                <svg source={mediaAssets['gallery/system.svg']} width={16} height={16} color={selectedTheme() === 'system' ? theme().primary : theme().text} />
                <text color={selectedTheme() === 'system' ? theme().primary : theme().text}>System</text>
              </row>
            </Button>
          </ButtonGroup>
        </Popover>
      </row>
      <scrollView id="gallery-content" width="100%" height="100%" grow={1} scrollY={true}
        scrollbarSide="left" scrollbarWidth={3} scrollbarThumbColor={theme().border} scrollbarHoverColor={theme().textMuted}>
        <column width="100%" padding={24} gap={16}>{content()}</column>
      </scrollView>
    </column>
  </row>
}
