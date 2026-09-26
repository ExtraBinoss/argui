/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import {
  Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText, InputField, Popover,
  type WidgetTheme,
} from '@argui/widgets/react'
import { mediaAssets } from '../../assets.generated'

const documents = ['Archive', 'Report', 'Snooze', 'Search', 'Layouting']

/** Demonstrates joined actions, a split menu, search, vertical flow, and RTL. */
export function ButtonGroupPage(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [action, setAction] = useState('Choose an action')
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<string[]>([])
  const search = () => setResults(documents.filter((item) => item.toLowerCase().includes(query.toLowerCase())))

  return <column id="button-group-page" width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>ButtonGroup</text>
    <text color={theme.textMuted}>Compose Button, InputField, ButtonGroupSeparator and ButtonGroupText inside one labelled group. Tab still reaches each control.</text>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Related actions</text>
      <text color={theme.textMuted}>The shared border and separators keep three independent actions visually connected.</text>
      <ButtonGroup id="group-actions" accessibleName="Document actions">
        <Button variant="ghost" onClick={() => setAction('Archived')}>Archive</Button>
        <ButtonGroupSeparator />
        <Button variant="ghost" onClick={() => setAction('Reported')}>Report</Button>
        <ButtonGroupSeparator />
        <Button variant="ghost" onClick={() => setAction('Snoozed')}>Snooze</Button>
      </ButtonGroup>
      <text color={theme.text}>{action}</text>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Search with an input</text>
      <text color={theme.textMuted}>InputField and Button share one frame. Search filters a small local list when pressed.</text>
      <ButtonGroup id="group-search" accessibleName="Search examples">
        <InputField width={220} accessibleName="Search term" type="search" value={query}
          onValueChange={setQuery} onSubmit={search} placeholder="Search examples"
          leading={<svg source={mediaAssets['tabler/search.svg']} width={16} height={16} color={theme.textMuted} />} />
        <ButtonGroupSeparator />
        <Button variant="ghost" onClick={search}>Search</Button>
      </ButtonGroup>
      <text color={theme.text}>{results.length ? `Matches: ${results.join(', ')}` : 'Type a term, then press Search.'}</text>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Text and a split action</text>
      <text color={theme.textMuted}>A static label and Popover can be composed without a special ButtonGroup mode.</text>
      <ButtonGroup id="group-split" accessibleName="Send action">
        <ButtonGroupText>Send as</ButtonGroupText>
        <ButtonGroupSeparator />
        <Button variant="default" onClick={() => setAction('Sent now')}>Send</Button>
        <ButtonGroupSeparator />
        <Popover trigger="More" placement="bottomEnd" contentWidth={220} opaque={true}>
          <Button variant="ghost" onClick={() => setAction('Scheduled')}>Schedule</Button>
          <Button variant="ghost" onClick={() => setAction('Saved draft')}>Save draft</Button>
        </Popover>
      </ButtonGroup>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Vertical orientation</text>
      <ButtonGroup id="group-vertical" accessibleName="View modes" orientation="vertical" width={150}>
        <Button width="100%" contentAlign="start" variant="ghost" onClick={() => setAction('List view')}>List</Button>
        <ButtonGroupSeparator />
        <Button width="100%" contentAlign="start" variant="ghost" onClick={() => setAction('Grid view')}>Grid</Button>
      </ButtonGroup>
    </column>

    <column width="100%" gap={8}>
      <text color={theme.text} fontSize={16}>Right to left</text>
      <text color={theme.textMuted}>directionScope changes logical start and end while the same components remain usable.</text>
      <ButtonGroup id="group-rtl" accessibleName="إجراءات المستند" directionScope="rtl">
        <Button variant="ghost" onClick={() => setAction('أرشفة')}>أرشفة</Button>
        <ButtonGroupSeparator />
        <Button variant="ghost" onClick={() => setAction('تقرير')}>تقرير</Button>
        <ButtonGroupSeparator />
        <Button variant="ghost" onClick={() => setAction('تأجيل')}>تأجيل</Button>
      </ButtonGroup>
    </column>
  </column>
}
