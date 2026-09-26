import { VirtualList } from '@argui/widgets/solid'
import { useTheme } from '@argui/solid'
import type { WidgetTheme } from '@argui/widgets/solid'

const entries = Array.from({ length: 5000 }, (_, index) => ({
  id: `record-${index + 1}`,
  label: `Record ${index + 1}`,
}))

/** Shows native range-based rendering over a large stable list. */
export function VirtualListPage() {
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" height={520} gap={16}>
    <text color={theme().text} fontSize={24}>VirtualList</text>
    <text color={theme().textMuted}>Only the visible range is mounted. itemKey keeps item identity stable.</text>
    <VirtualList
      id="gallery-records"
      count={entries.length}
      height={430}
      estimate={36}
      itemKey={(index) => entries[index]!.id}
      renderItem={(index) => <row width="100%" height={36} padding={8} alignItems="center">
        <text color={theme().text}>{entries[index]!.label}</text>
      </row>}
    />
  </column>
}
