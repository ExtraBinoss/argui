/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../../../packages/widgets/src/react/tabs'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates controlled tabs, line styling, disabled tabs and vertical orientation. */
export function TabsPage(props: { theme: Palette }): ReactElement {
  const [accountTab, setAccountTab] = useState('profile')
  return <column width="fill" gap={24}>
    <text text={`Selected tab: ${accountTab}`} color={props.theme.muted} font_size={13} />
    <Tabs id="surface-a-tabs" theme={props.theme} value={accountTab} onValueChange={setAccountTab}
      label="Account settings">
      <TabsList label="Account settings" variant="line">
        <TabsTrigger value="profile">Profile</TabsTrigger>
        <TabsTrigger value="security">Security</TabsTrigger>
        <TabsTrigger value="billing" disabled>Billing</TabsTrigger>
      </TabsList>
      <TabsContent value="profile"><column gap={8}>
        <text text="Profile" color={props.theme.foreground} font_size={17} weight={600} />
        <text width="fill" text="Update the name and contact details associated with your workspace account." color={props.theme.muted} font_size={14} />
      </column></TabsContent>
      <TabsContent value="security"><column gap={8}>
        <text text="Security" color={props.theme.foreground} font_size={17} weight={600} />
        <text width="fill" text="Review sign-in methods and session settings for your account." color={props.theme.muted} font_size={14} />
      </column></TabsContent>
      <TabsContent value="billing"><text text="Billing" color={props.theme.foreground} font_size={14} /></TabsContent>
    </Tabs>
    <Tabs id="surface-a-tabs-vertical" theme={props.theme} defaultValue="overview"
      orientation="vertical" activationMode="manual" label="Project sections">
      <TabsList label="Project sections">
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="activity">Activity</TabsTrigger>
        <TabsTrigger value="members">Members</TabsTrigger>
      </TabsList>
      <TabsContent value="overview"><text width="fill" text="Project overview and current status." color={props.theme.foreground} font_size={14} /></TabsContent>
      <TabsContent value="activity"><text width="fill" text="Recent project activity." color={props.theme.foreground} font_size={14} /></TabsContent>
      <TabsContent value="members"><text width="fill" text="People who can access this project." color={props.theme.foreground} font_size={14} /></TabsContent>
    </Tabs>
    <text text="Use Left or Right for horizontal tabs and Up or Down for vertical tabs. The second list uses manual activation."
      width="fill" color={props.theme.muted} font_size={13} />
  </column>
}
