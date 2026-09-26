/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Button, type WidgetTheme } from '@argui/widgets/react'
import { mediaAssets } from '../../assets.generated'

/** Shows every button variant, a toggle icon, and optional press motion. */
export function ButtonPage(): ReactElement {
  const [clicks, setClicks] = useState(0)
  const [favorite, setFavorite] = useState(false)
  const [motion, setMotion] = useState(true)
  const theme = useTheme<WidgetTheme>()
  const activate = () => setClicks((current) => current + 1)
  const openProjectSite = () => {
    if (typeof window !== 'undefined') window.location.assign('https://extrabinoss.github.io/argui/')
  }
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Button</text>
    <text color={theme.textMuted}>Variants use the selected color family. Hover, focus, and press feedback stay native.</text>
    <row gap={8} wrap={true}>
      <Button id="button-default" onClick={activate}>Default</Button>
      <Button variant="outline" onClick={activate}>Outline</Button>
      <Button variant="secondary" onClick={activate}>Secondary</Button>
      <Button variant="ghost" onClick={activate}>Ghost</Button>
      <Button variant="destructive" onClick={activate}>Destructive</Button>
      <Button variant="link" onClick={openProjectSite}>Link</Button>
      <Button iconOnly accessibleName="Favorite" variant="ghost" pressed={favorite} onClick={() => setFavorite(current => !current)}>
        <svg source={mediaAssets['tabler/star.svg']} width={18} height={18} color={favorite ? theme.primary : theme.text} />
      </Button>
      <Button disabled onClick={activate}>Disabled</Button>
    </row>
    <text color={theme.text}>Sizes and icons</text>
    <row gap={8} wrap={true} alignItems="center">
      <Button size="xs" onClick={activate}>Extra small</Button>
      <Button size="sm" onClick={activate}>Small</Button>
      <Button size="default" onClick={activate}>Default</Button>
      <Button size="lg" onClick={activate}>Large</Button>
      <Button size="icon-sm" variant="outline" accessibleName="Search" onClick={activate}>
        <svg source={mediaAssets['tabler/search.svg']} width={14} height={14} color={theme.text} />
      </Button>
      <Button variant="outline" onClick={activate}>
        <row gap={6} alignItems="center">
          <svg source={mediaAssets['tabler/player-play.svg']} width={16} height={16} color={theme.text} />
          <text color={theme.text} fontSize={14}>With icon</text>
        </row>
      </Button>
      <Button rounded onClick={activate}>Rounded</Button>
    </row>
    <row directionScope="rtl" gap={8}>
      <Button variant="outline" onClick={activate}>إجراء بالعربية</Button>
    </row>
    <text color={theme.text}>Press motion</text>
    <row gap={8} alignItems="center" wrap={true}>
      <Button id="button-motion-toggle" variant="outline" pressed={motion} pressAnimation={false}
        onClick={() => setMotion(current => !current)}>{`Motion: ${motion ? 'On' : 'Off'}`}</Button>
      <Button id="button-motion-demo" size="lg" pressAnimation={motion} onClick={activate}>Click repeatedly</Button>
      <text color={theme.textMuted} text={motion ? 'Each press replays the bounce' : 'Press bounce is disabled'} />
    </row>
    <text color={theme.text} text={`Clicked ${clicks} times`} />
  </column>
}
