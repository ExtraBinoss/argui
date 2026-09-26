/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import {
  Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText, DirectionProvider,
  DropdownMenu, InputField, InputGroup, InputGroupAddon, InputGroupButton,
  InputGroupInput, Popover, Select, type Palette,
} from '@argui/widgets/react'

/** Shows joined groups and the compositions from the shadcn examples. */
export function ButtonGroupPage(props: { theme: Palette }): ReactElement {
  const [lastActionValue, setLastAction] = useState('None')
  const lastAction = () => lastActionValue
  const [queryValue, setQuery] = useState('')
  const query = () => queryValue
  const [addressValue, setAddress] = useState('')
  const address = () => addressValue
  const [alignValue, setAlign] = useState('left')
  const align = () => alignValue
  const action = (name: string) => () => setLastAction(name)
  return <column width="fill" gap={16}>
    <text width="fill" text="Related controls share a border while each action keeps its own focus stop."
      color={props.theme.muted} font_size={14} />

    <text text="Default" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Message actions">
      <Button id="group-archive" label="Archive" theme={props.theme} kind="outline" onClick={action('Archive')} />
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-report" label="Report" theme={props.theme} kind="outline" onClick={action('Report')} />
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-snooze" label="Snooze" theme={props.theme} kind="outline" onClick={action('Snooze')} />
    </ButtonGroup>

    <text text="Orientation" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Vertical media controls" orientation="vertical">
      <Button id="group-up" label="Move up" theme={props.theme} kind="outline" onClick={action('Move up')} />
      <ButtonGroupSeparator theme={props.theme} orientation="horizontal" />
      <Button id="group-down" label="Move down" theme={props.theme} kind="outline" onClick={action('Move down')} />
    </ButtonGroup>

    <text text="Size" color={props.theme.foreground} font_size={16} weight={600} />
    <row width="fill" wrap={true} gap={12} align_items="center">
      <ButtonGroup theme={props.theme} label="Small buttons">
        <Button id="group-small-one" label="One" size="sm" theme={props.theme} kind="outline" onClick={action('Small one')} />
        <ButtonGroupSeparator theme={props.theme} />
        <Button id="group-small-two" label="Two" size="sm" theme={props.theme} kind="outline" onClick={action('Small two')} />
      </ButtonGroup>
      <ButtonGroup theme={props.theme} label="Large buttons">
        <Button id="group-large-one" label="One" size="lg" theme={props.theme} kind="outline" onClick={action('Large one')} />
        <ButtonGroupSeparator theme={props.theme} />
        <Button id="group-large-two" label="Two" size="lg" theme={props.theme} kind="outline" onClick={action('Large two')} />
      </ButtonGroup>
    </row>

    <text text="Nested" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup label="Formatting groups" gap={8}>
      <ButtonGroup theme={props.theme} label="Text style">
        <Button id="group-bold" label="Bold" theme={props.theme} kind="outline" onClick={action('Bold')} />
        <ButtonGroupSeparator theme={props.theme} />
        <Button id="group-italic" label="Italic" theme={props.theme} kind="outline" onClick={action('Italic')} />
      </ButtonGroup>
      <ButtonGroup theme={props.theme} label="Alignment">
        <Button id="group-left" label="Left" theme={props.theme} kind="outline" onClick={action('Align left')} />
        <ButtonGroupSeparator theme={props.theme} />
        <Button id="group-right" label="Right" theme={props.theme} kind="outline" onClick={action('Align right')} />
      </ButtonGroup>
    </ButtonGroup>

    <text text="Separator" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Display controls">
      <ButtonGroupText theme={props.theme} text="Zoom" />
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-zoom-out" label="−" theme={props.theme} kind="outline" onClick={action('Zoom out')} />
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-zoom-in" label="+" theme={props.theme} kind="outline" onClick={action('Zoom in')} />
    </ButtonGroup>

    <text text="Split" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Save options">
      <Button id="group-save" label="Save" theme={props.theme} kind="primary" onClick={action('Save')} />
      <ButtonGroupSeparator theme={props.theme} />
      <DropdownMenu id="group-save-more" theme={props.theme} triggerLabel="More" width={90}
        items={[{ type: 'item', id: 'save-draft', label: 'Save draft' },
          { type: 'item', id: 'save-copy', label: 'Save a copy' }]}
        onSelect={setLastAction} />
    </ButtonGroup>

    <text text="Input" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Search messages">
      <container width={240}>
        <InputField id="group-search" label="Search messages" theme={props.theme} value={query()}
          onChange={setQuery} placeholder="Search messages" />
      </container>
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-search-button" label="Search" theme={props.theme} kind="outline"
        onClick={() => setLastAction(`Search: ${query()}`)} />
    </ButtonGroup>

    <text text="Input Group" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Website address controls">
      <container width={280}>
        <InputGroup theme={props.theme} label="Website address">
          <InputGroupInput id="group-address" label="Website address" theme={props.theme}
            value={address()} onChange={setAddress} placeholder="example.com" />
          <InputGroupAddon theme={props.theme} align="inline-end">
            <InputGroupButton id="group-address-clear" label="Clear address" theme={props.theme}
              onClick={() => setAddress('')} size="icon">
              <text text="×" color={props.theme.foreground} font_size={17} />
            </InputGroupButton>
          </InputGroupAddon>
        </InputGroup>
      </container>
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-address-go" label="Go" theme={props.theme} kind="outline"
        onClick={() => setLastAction(`Go: ${address()}`)} />
    </ButtonGroup>

    <text text="Dropdown Menu" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Export">
      <Button id="group-export" label="Export" theme={props.theme} kind="outline" onClick={action('Export')} />
      <ButtonGroupSeparator theme={props.theme} />
      <DropdownMenu id="group-export-menu" theme={props.theme} triggerLabel="Options"
        items={[{ type: 'item', id: 'export-pdf', label: 'PDF' },
          { type: 'item', id: 'export-csv', label: 'CSV' }]}
        onSelect={setLastAction} />
    </ButtonGroup>

    <text text="Select" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Alignment controls">
      <Select id="group-align" label="Alignment" theme={props.theme} width={190}
        options={['left', 'center', 'right']} value={align()} onChange={setAlign} />
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-apply" label="Apply" theme={props.theme} kind="outline"
        onClick={() => setLastAction(`Align ${align()}`)} />
    </ButtonGroup>

    <text text="Popover" color={props.theme.foreground} font_size={16} weight={600} />
    <ButtonGroup theme={props.theme} label="Share controls">
      <Button id="group-copy" label="Copy link" theme={props.theme} kind="outline" onClick={action('Copy link')} />
      <ButtonGroupSeparator theme={props.theme} />
      <Popover id="group-share" label="Share" theme={props.theme} opaque width={230} closeLabel={false}>
        <text text="Share this page with your team." color={props.theme.foreground} font_size={13} />
      </Popover>
    </ButtonGroup>

    <text text="RTL" color={props.theme.foreground} font_size={16} weight={600} />
    <DirectionProvider direction="rtl">
      <ButtonGroup theme={props.theme} label="Right to left controls">
        <Button id="group-rtl-one" label="الأول" theme={props.theme} kind="outline" onClick={action('First RTL')} />
        <ButtonGroupSeparator theme={props.theme} />
        <Button id="group-rtl-two" label="الثاني" theme={props.theme} kind="outline" onClick={action('Second RTL')} />
      </ButtonGroup>
    </DirectionProvider>

    <text text={`Last action: ${lastAction()}`} color={props.theme.muted} font_size={12} />
  </column>
}
