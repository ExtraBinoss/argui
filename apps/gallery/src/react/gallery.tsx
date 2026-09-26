/** @jsxImportSource @argui/react */
import { useEffect, useState, type ReactElement } from 'react'
import type { ThemeRuntime } from '@argui/host'
import { ThemeScope, useTheme, useThemeSnapshot } from '@argui/react'
import { Button, ButtonGroup, ButtonGroupSeparator, InputField, Popover, ScrollShadow, type WidgetTheme } from '@argui/widgets/react'
import { ButtonPage } from './button-page'
import { ButtonGroupPage } from './button-group-page'
import { CheckboxPage } from './checkbox-page'
import { SwitchPage } from './switch-page'
import { InputFieldPage } from './input-field-page'
import { SelectPage } from './select-page'
import { TabsPage } from './tabs-page'
import { SliderPage } from './slider-page'
import { ProgressPage } from './progress-page'
import { PopoverPage } from './popover-page'
import { VirtualListPage } from './virtual-list-page'
import { ScrollbarPage } from './scrollbar-page'
import { LayoutScenarios } from './layout-scenarios'
import { AnimationPage } from './animation-page'
import { ExpressivePage } from './expressive-page'
import { mediaAssets } from '../../assets.generated'
import { applyGalleryTheme, colorFamilies, familySwatch, watchSystemFamily, type ColorFamily } from '../theme-colors'

const pages = [
  { id: 'button', label: 'Button', category: 'Widgets' },
  { id: 'button-group', label: 'ButtonGroup', category: 'Widgets' },
  { id: 'checkbox', label: 'Checkbox', category: 'Widgets' },
  { id: 'switch', label: 'Switch', category: 'Widgets' },
  { id: 'input-field', label: 'InputField', category: 'Widgets' },
  { id: 'select', label: 'Select', category: 'Widgets' },
  { id: 'tabs', label: 'Tabs', category: 'Widgets' },
  { id: 'slider', label: 'Slider', category: 'Widgets' },
  { id: 'progress', label: 'Progress', category: 'Widgets' },
  { id: 'popover', label: 'Popover', category: 'Widgets' },
  { id: 'virtual-list', label: 'VirtualList', category: 'Widgets' },
  { id: 'layouting', label: 'Layouting', category: 'Examples' },
  { id: 'scrollbars', label: 'Scrollbars', category: 'Examples' },
  { id: 'animation', label: 'Animation', category: 'Examples' },
  { id: 'expressive', label: 'Expressive UI', category: 'Examples' },
] as const

type PageId = typeof pages[number]['id']

/** Renders a searchable sidebar, one widget page, and a compact settings popover. */
export function Gallery(props: { runtime: ThemeRuntime<WidgetTheme> }): ReactElement {
  const [page, setPage] = useState<PageId>('button')
  const [search, setSearch] = useState('')
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [family, setFamily] = useState<ColorFamily>('Neutral')
  const theme = useTheme<WidgetTheme>()
  const snapshot = useThemeSnapshot<WidgetTheme>()
  const selectedTheme = snapshot.variant ?? 'system'
  useEffect(() => watchSystemFamily(props.runtime, () => family), [props.runtime, family])
  const chooseFamily = (next: ColorFamily) => {
    setFamily(next)
    applyGalleryTheme(props.runtime, next, selectedTheme as 'light' | 'dark' | 'system')
  }
  const visiblePages = (category: 'Widgets' | 'Examples') => pages.filter((entry) => entry.category === category && entry.label.toLowerCase().includes(search.trim().toLowerCase()))
  const Content = page === 'button' ? ButtonPage
    : page === 'button-group' ? ButtonGroupPage
    : page === 'checkbox' ? CheckboxPage
    : page === 'switch' ? SwitchPage
    : page === 'input-field' ? InputFieldPage
    : page === 'select' ? SelectPage
    : page === 'tabs' ? TabsPage
    : page === 'slider' ? SliderPage
    : page === 'progress' ? ProgressPage
    : page === 'popover' ? PopoverPage
    : page === 'virtual-list' ? VirtualListPage : undefined

  return <row width="100%" height="100%" background={theme.surface}>
    <column id="gallery-sidebar" width={256} height="100%" shrink={0} padding={{ top: 16 }} gap={16} background={theme.sidebar}>
      <container padding={{ start: 24, end: 16 }}><text color={theme.text} fontSize={20}>Argui</text></container>
      <container padding={{ start: 16, end: 16 }}><InputField id="gallery-search" accessibleName="Search gallery" type="search"
        placeholder="Search gallery" value={search} onValueChange={setSearch}
        leading={<svg source={mediaAssets['tabler/search.svg']} width={16} height={16} color={theme.textMuted} />} /></container>
      <ThemeScope<WidgetTheme> overrides={{ ghostHover: theme.sidebarAccent }}>
        <rectangle id="gallery-navigation-frame" width="100%" grow={1} minHeight={0} background={theme.sidebar} clip={true}>
          <ScrollShadow id="gallery-navigation" role="navigation" accessibleName="Gallery pages"
            width="100%" height="100%"
            scrollbarSide="left" scrollbarWidth={3} scrollbarEndInset={0} scrollbarThumbColor={theme.border} scrollbarHoverColor={theme.textMuted}>
            <column width="100%" padding={{ start: 16, end: 16 }} gap={12}>
              {(['Widgets', 'Examples'] as const).map((category) => visiblePages(category).length > 0 && <column key={category} width="100%" gap={4}>
                <container padding={{ start: 8, bottom: 4 }}><text color={theme.textMuted} fontSize={12}>{category}</text></container>
                {visiblePages(category).map((entry) => <Button
                  key={entry.id}
                  id={`page-${entry.id}`}
                  width="100%"
                  contentAlign="start"
                  variant="ghost"
                  pressed={page === entry.id}
                  onClick={() => setPage(entry.id)}
                ><text color={page === entry.id ? theme.sidebarPrimary : theme.sidebarForeground}>{entry.label}</text></Button>)}
              </column>)}
            </column>
          </ScrollShadow>
        </rectangle>
      </ThemeScope>
    </column>
    <column width="100%" height="100%" grow={1} minWidth={0}>
      <row width="100%" padding={{ top: 12, right: 16, bottom: 4, left: 16 }} justifyContent="end">
        <Popover id="gallery-settings" trigger="Settings" placement="bottomEnd" contentWidth={300} blur={true}
          open={settingsOpen} onOpenChange={setSettingsOpen}
          leading={<svg source={mediaAssets['gallery/settings.svg']} width={16} height={16} color={settingsOpen ? theme.primary : theme.text} />}>
          <text color={theme.text} fontSize={14}>Appearance</text>
          <ButtonGroup accessibleName="Theme">
            <Button id="theme-light" pressed={selectedTheme === 'light'} variant="ghost"
              onClick={() => applyGalleryTheme(props.runtime, family, 'light')}>
              <row gap={6} alignItems="center">
                <svg source={mediaAssets['gallery/sun.svg']} width={16} height={16} color={selectedTheme === 'light' ? theme.primary : theme.text} />
                <text color={selectedTheme === 'light' ? theme.primary : theme.text}>Light</text>
              </row>
            </Button>
            <ButtonGroupSeparator />
            <Button id="theme-dark" pressed={selectedTheme === 'dark'} variant="ghost"
              onClick={() => applyGalleryTheme(props.runtime, family, 'dark')}>
              <row gap={6} alignItems="center">
                <svg source={mediaAssets['gallery/moon.svg']} width={16} height={16} color={selectedTheme === 'dark' ? theme.primary : theme.text} />
                <text color={selectedTheme === 'dark' ? theme.primary : theme.text}>Dark</text>
              </row>
            </Button>
            <ButtonGroupSeparator />
            <Button id="theme-system" pressed={selectedTheme === 'system'} variant="ghost"
              onClick={() => applyGalleryTheme(props.runtime, family, 'system')}>
              <row gap={6} alignItems="center">
                <svg source={mediaAssets['gallery/system.svg']} width={16} height={16} color={selectedTheme === 'system' ? theme.primary : theme.text} />
                <text color={selectedTheme === 'system' ? theme.primary : theme.text}>System</text>
              </row>
            </Button>
          </ButtonGroup>
          <text color={theme.text} fontSize={14}>Color family</text>
          <scrollView id="gallery-color-list" width="100%" height={228} scrollY={true} scrollbarWidth={3}
            scrollbarThumbColor={theme.border} scrollbarHoverColor={theme.textMuted}>
            <column width="100%" gap={4}>
              {colorFamilies.map((entry) => <Button key={entry} id={`color-${entry.toLowerCase()}`}
                width="100%" variant="ghost" pressed={family === entry} contentAlign="start"
                onClick={() => chooseFamily(entry)}>
                <row gap={6} alignItems="center">
                  <rectangle width={12} height={12} radii={6} background={familySwatch(entry, snapshot.resolvedVariant === 'dark')} />
                  <text color={theme.text} fontSize={12}>{entry}</text>
                </row>
              </Button>)}
            </column>
          </scrollView>
        </Popover>
      </row>
      <scrollView id="gallery-content" width="100%" height="100%" grow={1} scrollY={true}
        scrollbarSide="left" scrollbarWidth={3} scrollbarThumbColor={theme.border} scrollbarHoverColor={theme.textMuted}>
        <column width="100%" padding={24} gap={16}>
          {page === 'layouting'
            ? <column width="100%" gap={16}><text color={theme.text} fontSize={24}>Layouting</text><LayoutScenarios /></column>
            : page === 'scrollbars' ? <ScrollbarPage />
            : page === 'animation' ? <AnimationPage />
            : page === 'expressive' ? <ExpressivePage />
            : Content && <Content />}
        </column>
      </scrollView>
    </column>
  </row>
}
