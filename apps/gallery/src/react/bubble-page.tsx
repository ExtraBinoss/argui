/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Bubble, BubbleContent, BubbleGroup, BubbleReactions, Button, type Palette } from '@argui/widgets/react'

/** Shows incoming and outgoing bubble variants with a working reaction. */
export function BubblePage(props: { theme: Palette }): ReactElement {
  const [liked, setLiked] = useState(false)
  return <column width="fill" gap={16}>
    <text text="Bubbles distinguish participants and keep reactions beside the message."
      color={props.theme.muted} font_size={14} />
    <BubbleGroup label="Conversation preview">
      <Bubble theme={props.theme} label="Incoming message" variant="secondary"
        content={<BubbleContent><text text="Can we ship the gallery today?"
          color={props.theme.foreground} font_size={14} /></BubbleContent>} />
      <Bubble theme={props.theme} label="Outgoing message" align="end"
        content={<BubbleContent><text text="The native widgets are ready for review."
          color={props.theme.accentText} font_size={14} /></BubbleContent>}
        reactions={<BubbleReactions theme={props.theme} label="Message reactions">
          <Button id="bubble-like" label={liked ? 'Liked · 1' : 'Like'} theme={props.theme}
            kind={liked ? 'primary' : 'quiet'} size="xs" onClick={() => setLiked((value) => !value)} />
        </BubbleReactions>} />
      <Bubble theme={props.theme} label="Outlined note" variant="outline"
        content={<BubbleContent><text text="A separate note uses the outline treatment."
          color={props.theme.foreground} font_size={14} /></BubbleContent>} />
      <Bubble theme={props.theme} label="Destructive notice" variant="destructive"
        content={<BubbleContent><text text="This action needs confirmation."
          color={props.theme.destructive} font_size={14} /></BubbleContent>} />
    </BubbleGroup>
  </column>
}
