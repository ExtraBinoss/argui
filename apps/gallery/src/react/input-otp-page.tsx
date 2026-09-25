/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactFieldLabel as FieldLabel } from '../../../../packages/widgets/src/react/field'
import { ReactInputOTP as InputOTP, ReactInputOTPGroup as InputOTPGroup, ReactInputOTPSeparator as InputOTPSeparator, ReactInputOTPSlot as InputOTPSlot } from '../../../../packages/widgets/src/react/input-otp'

/** Demonstrates controlled digit entry, segmented groups, and an uncontrolled code. */
export function InputOtpPage(props: { theme: Palette }): ReactElement {
  const [code, setCode] = useState('')
  return <column width="fill" gap={14}>
    <text width="fill" text="One accessible native editor supplies digit filtering and keyboard editing; the slots are visual segments."
      color={props.theme.muted} font_size={13} />
    <InputOTP id="foundation-h-otp" label="Verification code" theme={props.theme} maxLength={6}
      value={code} onChange={setCode} required description="Enter the six-digit code from your authenticator.">
      <InputOTPGroup>
        <InputOTPSlot index={0} /><InputOTPSlot index={1} /><InputOTPSlot index={2} />
      </InputOTPGroup>
      <InputOTPSeparator theme={props.theme} />
      <InputOTPGroup>
        <InputOTPSlot index={3} /><InputOTPSlot index={4} /><InputOTPSlot index={5} />
      </InputOTPGroup>
    </InputOTP>
    <text text={code.length === 6 ? `Code ready: ${code}` : `${code.length} of 6 digits entered`}
      color={code.length === 6 ? props.theme.accent : props.theme.muted} font_size={13} />
    <FieldLabel text="Saved code (read only)" theme={props.theme} htmlFor="foundation-h-saved-otp" />
    <InputOTP id="foundation-h-saved-otp" label="Saved verification code" labelledBy="foundation-h-saved-otp-label"
      theme={props.theme} maxLength={6}
      defaultValue="204813" readOnly>
      <InputOTPGroup>
        <InputOTPSlot index={0} /><InputOTPSlot index={1} /><InputOTPSlot index={2} />
        <InputOTPSlot index={3} /><InputOTPSlot index={4} /><InputOTPSlot index={5} />
      </InputOTPGroup>
    </InputOTP>
  </column>
}
