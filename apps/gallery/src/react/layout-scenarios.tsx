/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Button, type WidgetTheme } from '@argui/widgets/react'

/** Shows the same fixed, flexible, scroll, grid, and logical-inset cases in both adapters. */
export function LayoutScenarios(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [expanded, setExpanded] = useState(false)
  const gridColumns = [
    { repeat: { count: 'autoFit' as const, tracks: [{ minmax: { min: 140, max: { fr: 1 } } }] } },
  ]

  return <column id="layout-scenarios" width="100%" gap={12}>
    <text color={theme.text} fontSize={20}>Layout scenarios</text>
    <text color={theme.textMuted}>Change the available width, scroll the bounded list, and hover the surfaces. Each scene uses the same native layout rules as an application.</text>

    <text color={theme.text} fontSize={16}>Live width · percentage and grow</text>
    <text color={theme.textMuted}>Press the button to resize the parent. The muted card keeps 30% of its width while the blue card takes the remaining space. The sentence is inside the blue rectangle so its wrapping changes with the available width.</text>
    <Button id="layout-resize" variant="secondary" onClick={() => setExpanded((value) => !value)}>{expanded ? 'Narrow to 320 px' : 'Expand to 520 px'}</Button>
    <row id="layout-live-row" width={expanded ? 520 : 320} maxWidth="100%" gap={8}>
      <rectangle id="layout-live-percent" width="30%" shrink={0} minHeight={80} padding={8} radii={6} background={theme.surfaceHover}>
        <text color={theme.text}>30% of parent</text>
      </rectangle>
      <rectangle id="layout-live-grow" grow={1} minWidth={0} minHeight={80} padding={8} radii={6} background={theme.primary}>
        <text color={theme.primaryForeground}>This sentence lives inside the growing rectangle and wraps as the available width changes.</text>
      </rectangle>
    </row>

    <text color={theme.text} fontSize={16}>Flex: fixed width plus remaining space</text>
    <text color={theme.textMuted}>The first child stays 100 px wide; grow=1 assigns the rest to the second child.</text>
    <row id="layout-fixed-grow-320" width={320} maxWidth="100%" height={48} gap={8}>
      <rectangle id="layout-fixed-320" width={100} shrink={0} padding={8} radii={6} background={theme.surfaceHover} hoverBackground={theme.border} transitionMs={140}>
        <text color={theme.text}>Fixed 100</text>
      </rectangle>
      <rectangle id="layout-grow-320" grow={1} minWidth={0} padding={8} radii={6} background={theme.primary} hoverBackground={theme.focusRing} transitionMs={140}>
        <text color={theme.primaryForeground}>Grow remainder</text>
      </rectangle>
    </row>
    <row id="layout-fixed-grow-640" width={640} maxWidth="100%" height={48} gap={8}>
      <rectangle id="layout-fixed-640" width={100} shrink={0} padding={8} radii={6} background={theme.surfaceHover} hoverBackground={theme.border} transitionMs={140}>
        <text color={theme.text}>Fixed 100</text>
      </rectangle>
      <rectangle id="layout-grow-640" grow={1} minWidth={0} padding={8} radii={6} background={theme.primary} hoverBackground={theme.focusRing} transitionMs={140}>
        <text color={theme.primaryForeground}>Grow remainder</text>
      </rectangle>
    </row>

    <text color={theme.text} fontSize={16}>Long text and minWidth</text>
    <text color={theme.textMuted}>minWidth=0 lets a flex item shrink so this sentence wraps inside the remaining space.</text>
    <row id="layout-long-text-row" width={320} maxWidth="100%" gap={8} alignItems="center">
      <rectangle id="layout-long-text-fixed" width={76} height={32} shrink={0} radii={6} background={theme.border} />
      <rectangle grow={1} minWidth={0} padding={6} radii={6} background={theme.surfaceHover}>
        <text id="layout-long-text" color={theme.text}>A longer sentence wraps inside this narrow rectangle.</text>
      </rectangle>
    </row>
    <text color={theme.textMuted}>A fixed 420 px text width in the same 320 px row makes the intrinsic overflow visible.</text>
    <row id="layout-overflow-row" width={320} maxWidth="100%" gap={8}>
      <rectangle width={76} height={32} shrink={0} radii={6} background={theme.border} />
      <rectangle grow={1} clip={true} padding={6} radii={6} background={theme.surfaceHover}>
        <text id="layout-overflow-text" width={420} color={theme.text}>Long content beyond the row</text>
      </rectangle>
    </row>

    <text color={theme.text} fontSize={16}>Bounded scrolling</text>
    <text color={theme.textMuted}>The viewport is 96 px tall; six 28 px rows require vertical scrolling.</text>
    <scrollView id="layout-bounded-scroll" width={320} maxWidth="100%" height={96} scrollY={true}>
      <column width="100%" gap={4}>
        {Array.from({ length: 6 }, (_, index) => <rectangle
          key={index}
          id={`layout-scroll-row-${index}`}
          width="100%"
          height={28}
          shrink={0}
          padding={6}
          radii={4}
          background={index % 2 ? theme.surface : theme.surfaceHover}
          hoverBackground={theme.border}
          transitionMs={120}
        ><text color={theme.text} text={`Scrollable row ${index + 1}`} /></rectangle>)}
      </column>
    </scrollView>

    <text color={theme.text} fontSize={16}>Adaptive grid</text>
    <text color={theme.textMuted}>autoFit repeats tracks with a 140 px minimum and shares extra space with fr=1.</text>
    <grid id="layout-auto-fit-grid-320" width={320} maxWidth="100%" gap={8} gridColumns={gridColumns}>
      {Array.from({ length: 4 }, (_, index) => <rectangle
        key={index}
        id={`layout-grid-320-${index}`}
        height={36}
        background={theme.surfaceHover}
        hoverBackground={theme.border}
        radii={6}
        transitionMs={120}
      ><text color={theme.text} text={`Card ${index + 1}`} /></rectangle>)}
    </grid>
    <grid id="layout-auto-fit-grid-640" width={640} maxWidth="100%" gap={8} gridColumns={gridColumns}>
      {Array.from({ length: 4 }, (_, index) => <rectangle
        key={index}
        id={`layout-grid-640-${index}`}
        height={36}
        background={theme.surfaceHover}
        hoverBackground={theme.border}
        radii={6}
        transitionMs={120}
      ><text color={theme.text} text={`Card ${index + 1}`} /></rectangle>)}
    </grid>

    <text color={theme.text} fontSize={16}>Logical inset in right-to-left direction</text>
    <text color={theme.textMuted}>The marker uses start=12 and top=8; start resolves from the right edge in this scope.</text>
    <container id="layout-rtl-inset" width={240} maxWidth="100%" height={80} position="relative" directionScope="rtl" background={theme.surfaceHover}>
      <rectangle id="layout-rtl-marker" width={120} height={28} position="absolute" inset={{ start: 12, top: 8 }} padding={4} radii={6} background={theme.primary} hoverBackground={theme.focusRing} transitionMs={120}>
        <text color={theme.primaryForeground}>start 12 / top 8</text>
      </rectangle>
    </container>
  </column>
}
