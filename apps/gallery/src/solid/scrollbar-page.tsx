import { createSignal } from 'solid-js'
import { useTheme } from '@argui/solid'
import { Button, ButtonGroup, ButtonGroupSeparator, type WidgetTheme } from '@argui/widgets/solid'

const rows = Array.from({ length: 18 }, (_, index) => `Scrollable item ${String(index + 1).padStart(2, '0')}`)

/** Demonstrates native scrollbar geometry, state colors, and scroll momentum. */
export function ScrollbarPage() {
  const theme = useTheme<WidgetTheme>()
  const [width, setWidth] = createSignal(6)
  const [side, setSide] = createSignal<'left' | 'right'>('right')
  const [grow, setGrow] = createSignal(true)
  const [momentum, setMomentum] = createSignal(0.35)

  return <column width="100%" gap={20}>
    <column width="100%" gap={6}>
      <text color={theme().text} fontSize={24}>Scrollbars</text>
      <text color={theme().textMuted}>Customize the native track and thumb. Hover the thumb to widen it; drag to see its pressed state.</text>
    </column>

    <rectangle width="100%" maxWidth={680} padding={16} radii={12} background={theme().card}
      border={{ width: 1, color: theme().border }}>
      <column width="100%" gap={14}>
        <text color={theme().text} fontSize={16}>Interactive scrollbar</text>
        <row width="100%" wrap={true} gap={12} alignItems="center">
          <column gap={5}>
            <text color={theme().textMuted} fontSize={12}>Width</text>
            <ButtonGroup accessibleName="Scrollbar width">
              {[4, 6, 8].map((value) => <Button variant="ghost" pressed={width() === value}
                onClick={() => setWidth(value)}>{`${value} px`}</Button>)}
            </ButtonGroup>
          </column>
          <column gap={5}>
            <text color={theme().textMuted} fontSize={12}>Side</text>
            <ButtonGroup accessibleName="Scrollbar side">
              <Button variant="ghost" pressed={side() === 'left'} onClick={() => setSide('left')}>Left</Button>
              <ButtonGroupSeparator />
              <Button variant="ghost" pressed={side() === 'right'} onClick={() => setSide('right')}>Right</Button>
            </ButtonGroup>
          </column>
          <column gap={5}>
            <text color={theme().textMuted} fontSize={12}>Hover width</text>
            <Button variant="outline" pressed={grow()} onClick={() => setGrow(!grow())}>{grow() ? '+4 px on hover' : 'No growth'}</Button>
          </column>
        </row>
        <row width="100%" wrap={true} gap={12} alignItems="center">
          <column gap={5}>
            <text color={theme().textMuted} fontSize={12}>Momentum after scrolling</text>
            <ButtonGroup accessibleName="Scroll momentum">
              <Button variant="ghost" pressed={momentum() === 0} onClick={() => setMomentum(0)}>Direct</Button>
              <ButtonGroupSeparator />
              <Button variant="ghost" pressed={momentum() === 0.35} onClick={() => setMomentum(0.35)}>Light</Button>
              <ButtonGroupSeparator />
              <Button variant="ghost" pressed={momentum() === 0.7} onClick={() => setMomentum(0.7)}>Glide</Button>
            </ButtonGroup>
          </column>
        </row>
        <row width="100%" wrap={true} gap={14} alignItems="center">
          <row gap={5} alignItems="center"><rectangle width={10} height={10} radii={5} background={theme().chart2} /><text color={theme().textMuted} fontSize={12}>Rest</text></row>
          <row gap={5} alignItems="center"><rectangle width={10} height={10} radii={5} background={theme().primary} /><text color={theme().textMuted} fontSize={12}>Hover</text></row>
          <row gap={5} alignItems="center"><rectangle width={10} height={10} radii={5} background={theme().foreground} /><text color={theme().textMuted} fontSize={12}>Dragging</text></row>
        </row>
        <scrollView id="gallery-custom-scrollbar" width="100%" height={232} scrollY={true}
          scrollMomentum={momentum()} scrollbarSide={side()} scrollbarWidth={width()}
          scrollbarHoverWidth={grow() ? width() + 4 : width()}
          scrollbarTrackColor={theme().muted} scrollbarThumbColor={theme().chart2}
          scrollbarHoverColor={theme().primary} scrollbarPressedColor={theme().foreground}>
          <column width="100%" gap={4}>
            {rows.map((label, index) => <rectangle width="100%" height={38} shrink={0}
              padding={{ start: 12, end: 16 }} radii={6}
              background={index % 2 === 0 ? theme().card : theme().muted}>
              <row width="100%" height="100%" alignItems="center" gap={10}>
                <text color={theme().textMuted} fontSize={12}>{String(index + 1).padStart(2, '0')}</text>
                <text color={theme().text} fontSize={14}>{label}</text>
              </row>
            </rectangle>)}
          </column>
        </scrollView>
        <text color={theme().textMuted} fontSize={12}>The track and thumb colors change in the renderer. Hover width animates for 140 ms and changes the draggable geometry with it.</text>
      </column>
    </rectangle>
  </column>
}
