/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import type { WidgetPageName } from '../widget-pages'
import { AlertPage } from './alert-page'
import { BadgePage } from './badge-page'
import { CardPage } from './card-page'
import { AvatarPage } from './avatar-page'
import { AspectRatioPage } from './aspect-ratio-page'
import { SeparatorPage } from './separator-page'
import { SkeletonPage } from './skeleton-page'
import { SpinnerPage } from './spinner-page'
import { KbdPage } from './kbd-page'
import { LabelPage } from './label-page'
import { TextareaPage } from './textarea-page'
import { NativeSelectPage } from './native-select-page'
import { ButtonGroupPage } from './button-group-page'
import { EmptyPage } from './empty-page'
import { ItemPage } from './item-page'
import { ProgressPage } from './progress-page'
import { SliderPage } from './slider-page'
import { SwitchPage } from './switch-page'
import { CheckboxPage } from './checkbox-page'
import { RadioGroupPage } from './radio-group-page'
import { TogglePage } from './toggle-page'
import { AccordionPage } from './accordion-page'
import { CollapsiblePage } from './collapsible-page'
import { TabsPage } from './tabs-page'
import { FieldPage } from './field-page'
import { InputGroupPage } from './input-group-page'
import { InputOtpPage } from './input-otp-page'
import { TooltipPage } from './tooltip-page'
import { AlertDialogPage } from './alert-dialog-page'
import { DrawerPage } from './drawer-page'
import { SheetPage } from './sheet-page'
import { BreadcrumbPage } from './breadcrumb-page'
import { FormPage } from './form-page'
import { ToggleGroupPage } from './toggle-group-page'
import { ContextMenuPage } from './context-menu-page'
import { DropdownMenuPage } from './dropdown-menu-page'
import { MenubarPage } from './menubar-page'
import { HoverCardPage } from './hover-card-page'
import { ScrollAreaPage } from './scroll-area-page'
import { MarkerPage } from './marker-page'
import { CalendarPage } from './calendar-page'
import { CarouselPage } from './carousel-page'
import { ResizablePage } from './resizable-page'
import { BubblePage } from './bubble-page'
import { ComboboxPage } from './combobox-page'
import { CommandPage } from './command-page'
import { PaginationPage } from './pagination-page'
import { DirectionPage } from './direction-page'
import { NavigationMenuPage } from './navigation-menu-page'
import { SidebarPage } from './sidebar-page'
import { AttachmentPage } from './attachment-page'
import { MessagePage } from './message-page'
import { MessageScrollerPage } from './message-scroller-page'
import { ChartPage } from './chart-page'
import { TablePage } from './table-page'
import { QuestionnairePage } from './questionnaire-page'
import { SonnerPage } from './sonner-page'
import { ToastPage } from './toast-page'

/** Renders one React component demonstration selected from shared navigation. */
export function WidgetPage(props: { name: WidgetPageName; theme: Palette }): ReactElement {
  switch (props.name) {
    case 'Accordion': return <AccordionPage theme={props.theme} />
    case 'Alert Dialog': return <AlertDialogPage theme={props.theme} />
    case 'Alert': return <AlertPage theme={props.theme} />
    case 'Aspect Ratio': return <AspectRatioPage theme={props.theme} />
    case 'Attachment': return <AttachmentPage theme={props.theme} />
    case 'Avatar': return <AvatarPage theme={props.theme} />
    case 'Badge': return <BadgePage theme={props.theme} />
    case 'Breadcrumb': return <BreadcrumbPage theme={props.theme} />
    case 'Button Group': return <ButtonGroupPage theme={props.theme} />
    case 'Bubble': return <BubblePage theme={props.theme} />
    case 'Card': return <CardPage theme={props.theme} />
    case 'Calendar': return <CalendarPage theme={props.theme} />
    case 'Chart': return <ChartPage theme={props.theme} />
    case 'Carousel': return <CarouselPage theme={props.theme} />
    case 'Checkbox': return <CheckboxPage theme={props.theme} />
    case 'Collapsible': return <CollapsiblePage theme={props.theme} />
    case 'Combobox': return <ComboboxPage theme={props.theme} />
    case 'Command': return <CommandPage theme={props.theme} />
    case 'Context Menu': return <ContextMenuPage theme={props.theme} />
    case 'Direction': return <DirectionPage theme={props.theme} />
    case 'Empty': return <EmptyPage theme={props.theme} />
    case 'Drawer': return <DrawerPage theme={props.theme} />
    case 'Dropdown Menu': return <DropdownMenuPage theme={props.theme} />
    case 'Field': return <FieldPage theme={props.theme} />
    case 'Form': return <FormPage theme={props.theme} />
    case 'Hover Card': return <HoverCardPage theme={props.theme} />
    case 'Input Group': return <InputGroupPage theme={props.theme} />
    case 'Input OTP': return <InputOtpPage theme={props.theme} />
    case 'Item': return <ItemPage theme={props.theme} />
    case 'Kbd': return <KbdPage theme={props.theme} />
    case 'Label': return <LabelPage theme={props.theme} />
    case 'Menubar': return <MenubarPage theme={props.theme} />
    case 'Marker': return <MarkerPage theme={props.theme} />
    case 'Message': return <MessagePage theme={props.theme} />
    case 'Message Scroller': return <MessageScrollerPage theme={props.theme} />
    case 'Native Select': return <NativeSelectPage theme={props.theme} />
    case 'Navigation Menu': return <NavigationMenuPage theme={props.theme} />
    case 'Pagination': return <PaginationPage theme={props.theme} />
    case 'Progress': return <ProgressPage theme={props.theme} />
    case 'Questionnaire': return <QuestionnairePage theme={props.theme} />
    case 'Radio Group': return <RadioGroupPage theme={props.theme} />
    case 'Resizable': return <ResizablePage theme={props.theme} />
    case 'Scroll Area': return <ScrollAreaPage theme={props.theme} />
    case 'Sheet': return <SheetPage theme={props.theme} />
    case 'Sidebar': return <SidebarPage theme={props.theme} />
    case 'Separator': return <SeparatorPage theme={props.theme} />
    case 'Slider': return <SliderPage theme={props.theme} />
    case 'Skeleton': return <SkeletonPage theme={props.theme} />
    case 'Sonner': return <SonnerPage theme={props.theme} />
    case 'Spinner': return <SpinnerPage theme={props.theme} />
    case 'Switch': return <SwitchPage theme={props.theme} />
    case 'Tabs': return <TabsPage theme={props.theme} />
    case 'Table': return <TablePage theme={props.theme} />
    case 'Toast': return <ToastPage theme={props.theme} />
    case 'Textarea': return <TextareaPage theme={props.theme} />
    case 'Toggle': return <TogglePage theme={props.theme} />
    case 'Toggle Group': return <ToggleGroupPage theme={props.theme} />
    case 'Tooltip': return <TooltipPage theme={props.theme} />
  }
}
