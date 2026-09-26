import { VirtualList, type WidgetTheme } from '@argui/widgets/solid'
import { useTheme } from '@argui/solid'

const entries = Array.from({ length: 5000 }, (_, index) => ({
  id: `record-${index + 1}`,
  label: `Record ${index + 1}`,
}))
const sections = ['Overview', 'Components', 'Typography', 'Colors', 'Spacing', 'Motion', 'Keyboard', 'Accessibility', 'Testing', 'Release']
const categories = ['Design', 'Runtime', 'Motion', 'Text']

/** Compares a plain native scroll viewport with a headless virtualizer. */
export function VirtualListPage() {
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" gap={20}>
    <column width="100%" gap={6}>
      <text color={theme().text} fontSize={24}>Scrolling</text>
      <text color={theme().textMuted}>A regular viewport for short content, and a virtualizer for 5,000 rows. Scroll inside either panel.</text>
    </column>

    <rectangle width="100%" maxWidth={680} padding={16} radii={12} background={theme().card}
      border={{ width: 1, color: theme().border }}>
      <column width="100%" gap={12}>
        <row width="100%" alignItems="center" justifyContent="spaceBetween">
          <column gap={4}>
            <text color={theme().text} fontSize={16}>Native scrolling</text>
            <text color={theme().textMuted} fontSize={12}>Every row exists; the viewport clips and scrolls them.</text>
          </column>
          <text color={theme().textMuted} fontSize={12}>10 rows</text>
        </row>
        <scrollView id="gallery-scroll-example" width="100%" height={160} scrollY={true}
          scrollbarWidth={6} scrollbarThumbColor={theme().textMuted} scrollbarHoverColor={theme().text}>
          <column width="100%" gap={4}>
            {sections.map((section, index) => <rectangle width="100%" height={40} shrink={0} padding={{ start: 12, end: 16 }}
              radii={6} background={theme().muted} hoverBackground={theme().accent} transitionMs={120}>
              <row width="100%" height="100%" alignItems="center" gap={12}>
                <text color={theme().textMuted} fontSize={12}>{String(index + 1).padStart(2, '0')}</text>
                <text color={theme().text} fontSize={14}>{section}</text>
              </row>
            </rectangle>)}
          </column>
        </scrollView>
      </column>
    </rectangle>

    <rectangle width="100%" maxWidth={680} padding={16} radii={12} background={theme().card}
      border={{ width: 1, color: theme().border }}>
      <column width="100%" gap={12}>
        <row width="100%" alignItems="center" justifyContent="spaceBetween">
          <column gap={4}>
            <text color={theme().text} fontSize={16}>Virtualized scrolling</text>
            <text color={theme().textMuted} fontSize={12}>The same native scroll behavior; only the visible keyed rows are mounted.</text>
          </column>
          <text color={theme().textMuted} fontSize={12}>5,000 rows</text>
        </row>
        <VirtualList id="gallery-records" count={entries.length} width="100%" height={320}
          estimate={52} variable={false} overscan={8} scrollMomentum={0.35} scrollbarVisible={true}
          scrollbarWidth={6} scrollbarColor={theme().textMuted}
          itemKey={(index) => entries[index]!.id}
          renderItem={(index) => <row width="100%" height={52} alignItems="center" gap={12}
            padding={{ start: 12, end: 20 }} background={index % 2 === 0 ? theme().muted : theme().card}>
            <text color={theme().textMuted} fontSize={12}>{String(index + 1).padStart(4, '0')}</text>
            <text color={theme().text} fontSize={14}>{entries[index]!.label}</text>
            <container grow={1} />
            <text color={theme().textMuted} fontSize={12}>{categories[index % categories.length]}</text>
          </row>}
        />
      </column>
    </rectangle>
  </column>
}
