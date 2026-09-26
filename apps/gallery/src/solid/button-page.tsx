import { createSignal } from '@argui/solid'
import { useTheme } from '@argui/solid'
import { Button, type WidgetTheme } from '@argui/widgets/solid'
import { mediaAssets } from '../../assets.generated'

/** Shows every button variant, a toggle icon, and optional press motion. */
export function ButtonPage() {
  const [clicks, setClicks] = createSignal(0)
  const [favorite, setFavorite] = createSignal(false)
  const theme = useTheme<WidgetTheme>()
  const activate = () => setClicks(clicks() + 1)
  return <column width="100%" gap={16}>
    <text color={theme().text} fontSize={24}>Button</text>
    <text color={theme().textMuted}>Button text comes from children. Colors use the Neutral roles; hover and press feedback stay native.</text>
    <row gap={8} wrap={true}>
      <Button id="button-default" onClick={activate}>Default</Button>
      <Button variant="outline" onClick={activate}>Outline</Button>
      <Button variant="secondary" onClick={activate}>Secondary</Button>
      <Button variant="ghost" onClick={activate}>Ghost</Button>
      <Button variant="destructive" onClick={activate}>Destructive</Button>
      <Button variant="link" onClick={activate}>Link</Button>
      <Button iconOnly accessibleName="Favorite" variant="ghost" pressed={favorite()} onClick={() => setFavorite(!favorite())}>
        <svg source={mediaAssets['tabler/star.svg']} width={18} height={18} color={favorite() ? theme().primary : theme().text} />
      </Button>
      <Button disabled onClick={activate}>Disabled</Button>
    </row>
    <text color={theme().text}>Sizes and motion</text>
    <row gap={8} wrap={true} alignItems="center">
      <Button size="xs" onClick={activate}>Extra small</Button>
      <Button size="sm" onClick={activate}>Small</Button>
      <Button size="default" onClick={activate}>Default</Button>
      <Button size="lg" onClick={activate}>Large</Button>
      <Button variant="outline" pressAnimation={false} onClick={activate}>Motion off</Button>
    </row>
    <text color={theme().text} text={`Clicked ${clicks()} times`} />
  </column>
}
