import { docExampleSources } from './doc-example-sources.generated'

export type DocCode = { filename: string; code: string }
export type DocExample = { id: string; path: string; source: string }
export type DocSection = {
  id: string
  title: string
  paragraphs: string[]
  bullets?: string[]
  code?: DocCode
  note?: string
}

export type DocGuide = {
  slug: string
  category: 'Start here' | 'Essentials' | 'Advanced' | 'Architecture'
  level: 'Beginner' | 'Intermediate' | 'Advanced'
  minutes: number
  title: string
  description: string
  example: DocExample
  demoTitle: string
  sources: string[]
  sections: DocSection[]
}

const example = (id: string, filename: keyof typeof docExampleSources): DocExample => ({
  id,
  path: `app_examples/docs-examples/src/examples/${filename}.rs`,
  source: docExampleSources[filename],
})

const dependency = `[dependencies]
argui = { version = "0.2.1", features = ["widget-button"] }`

const launch = `use argui::{
    platform::{ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig},
    render::RendererConfig,
    runtime::{run_app, Context, Render},
    ui::Element,
};

#[derive(Default)]
struct App;

impl Render for App {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("Hello from Argui")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let identity = ApplicationIdentity::new(
        ApplicationId::new("dev.example.hello")?,
        "Hello Argui",
        IconSet::default(),
    );
    run_app(
        ApplicationConfig::new(identity, WindowConfig::default()),
        RendererConfig::default(),
        App,
        |event| println!("{event:?}"),
    )?;
    Ok(())
}`

const counter = `#[derive(Default)]
struct Counter {
    count: u32,
}

impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let palette = default_theme(cx.environment());
        let theme = palette.resolve(cx.environment().color_scheme);

        Element::column([
            Element::text(format!("Count: {}", self.count)),
            Button::new("increment", "Increment", theme.button())
                .build(),
        ])
        .gap(12.0)
        .on(cx.listener(EventType::Click, |app, event, cx| {
            if event.target_key() == Some("increment") {
                app.count = app.count.saturating_add(1);
                cx.notify();
            }
        }))
    }
}`

const layout = `let actions = Element::row([
    Button::new("save", "Save", theme.button()).build(),
    Button::new("cancel", "Cancel", theme.ghost_button()).build(),
])
.flex_wrap(FlexWrap::Wrap)
.gap(8.0);

Element::column([
    Element::text("Account settings"),
    form,
    actions,
])
.width(percent(1.0))
.max_width(length(720.0))
.padding(Sides::length(24.0))
.gap(20.0)`

const listener = `let save = Button::new("save", "Save", theme.button())
    .build()
    .on(cx.listener(EventType::Click, |model, event, cx| {
        debug_assert_eq!(event.target_key(), Some("save"));
        model.saved = true;
        cx.notify();
    }));`

const tasks = `fn search(&mut self, cx: &mut Context<Self>) -> Result<(), TaskError> {
    let query = self.query.clone();
    cx.spawn_latest(&mut self.task, load_results(query), |model, result, cx| {
        match result {
            Ok(Ok(items)) => model.results = items,
            Ok(Err(error)) => model.error = Some(error),
            Err(error) => model.error = Some(error.to_string()),
        }
        cx.notify();
    })?;
    Ok(())
}`

const custom = `#[derive(Debug)]
struct Ruler {
    zoom: f32,
    color: Color,
}

impl CustomElement for Ruler {
    type State = Vec<f32>;

    fn create_state(&self) -> Self::State { Vec::new() }
    fn layout_revision(&self) -> u64 { u64::from(self.zoom.to_bits()) }
    fn paint_revision(&self) -> u64 {
        u64::from(u32::from_le_bytes(self.color.to_srgba8()))
    }

    fn layout(
        &self,
        _state: &mut Self::State,
        _cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        Ok(CustomMeasurement {
            size: Size::new(800.0 * self.zoom, 80.0),
            baseline: None,
        })
    }

    fn prepare(&self, ticks: &mut Self::State, size: Size) {
        let spacing = 40.0 * self.zoom;
        ticks.clear();
        ticks.extend((0..(size.width / spacing).ceil() as usize).map(|tick| tick as f32 * spacing));
    }

    fn paint(&self, ticks: &mut Self::State, cx: &mut CustomPaintContext<'_>) {
        for &x in ticks.iter() {
            cx.quad(
                Rect::new(Point::new(x, 0.0), Size::new(1.0, cx.bounds.size.height)),
                QuadStyle::solid(self.color),
            );
        }
    }
}`

export const docs: DocGuide[] = [
  {
    slug: 'start/installation',
    category: 'Start here',
    level: 'Beginner',
    minutes: 5,
    title: 'Install Argui',
    description:
      'Create a Rust project, select only the features you use, and prepare desktop and WebAssembly targets.',
    example: example('installation', 'installation'),
    demoTitle: 'Your first compiled Argui control',
    sources: ['Cargo.toml', 'crates/argui/Cargo.toml', 'README.md'],
    sections: [
      {
        id: 'requirements',
        title: 'What you need',
        paragraphs: [
          'Argui 0.2.1 requires Rust 1.89 or newer. The repository itself currently recommends a newer toolchain for contributors, while the workspace manifest remains the source of truth for the minimum supported Rust version.',
        ],
        bullets: [
          'Rust and Cargo',
          'A GPU and driver supported by WGPU',
          'wasm32-unknown-unknown plus wasm-pack for browser builds',
        ],
        note: 'Argui is experimental and its API can still change. Pin the version used by your application.',
      },
      {
        id: 'dependency',
        title: 'Add the dependency',
        paragraphs: [
          'The facade has no default feature bundle. Enable the smallest widget set that your application needs; this keeps optional integrations and their dependencies out of the build.',
        ],
        code: { filename: 'Cargo.toml', code: dependency },
      },
      {
        id: 'targets',
        title: 'Prepare the browser target',
        paragraphs: [
          'Desktop targets are selected by Cargo. WebAssembly uses the same retained UI and renderer through WebGPU.',
        ],
        code: {
          filename: 'Terminal',
          code: 'rustup target add wasm32-unknown-unknown\ncargo check --target wasm32-unknown-unknown',
        },
      },
    ],
  },
  {
    slug: 'start/first-window',
    category: 'Start here',
    level: 'Beginner',
    minutes: 8,
    title: 'Create your first window',
    description: 'Launch a native or browser window and render one retained Argui element.',
    example: example('first-window', 'first_window'),
    demoTitle: 'A compiled responsive application window',
    sources: [
      'crates/argui/examples/window.rs',
      'crates/argui-runtime/src/launch.rs',
      'docs/platform/application.md',
    ],
    sections: [
      {
        id: 'application',
        title: 'One model, one presentation',
        paragraphs: [
          'A small application implements Render. The runtime owns the event loop and asks the model for an Element tree when its presentation is dirty. The same entry path supports desktop and WebAssembly.',
        ],
        code: { filename: 'src/main.rs', code: launch },
      },
      {
        id: 'configuration',
        title: 'Configure identity and window',
        paragraphs: [
          'ApplicationIdentity gives the application a stable reverse-domain identifier, display name, and icons. WindowConfig documents each public field in cargo doc and your IDE, including logical size, decorations, transparency, canvas attachment, pointer behavior, safe areas, and initial focus.',
          'focus_on_launch defaults to true on native targets and false on WebAssembly. The Web default keeps an embedded canvas from taking focus and moving the surrounding page while it loads; clicks and touches still focus it normally.',
        ],
        note: 'Use with_focus_on_launch(true) for a full-page Web app that should accept keyboard input immediately. Leave it disabled for examples embedded in a scrollable site.',
      },
      {
        id: 'errors',
        title: 'Keep startup errors visible',
        paragraphs: [
          'Return a Result from main and report RuntimeEvent failures. Renderer, layout, and command failures are explicit events; production applications should send them to their logging or crash-reporting path.',
        ],
      },
    ],
  },
  {
    slug: 'start/elements',
    category: 'Start here',
    level: 'Beginner',
    minutes: 10,
    title: 'Compose an interface',
    description: 'Build rows, columns, text, and widgets as a typed Rust tree.',
    example: example('elements', 'elements'),
    demoTitle: 'Composition with real Argui buttons',
    sources: [
      'crates/argui/examples/layout.rs',
      'crates/argui-ui/src/element.rs',
      'crates/argui-widget-gallery/src/pages/buttons.rs',
    ],
    sections: [
      {
        id: 'tree',
        title: 'Elements describe the view',
        paragraphs: [
          'Element::row and Element::column accept child elements. Builder methods apply typed layout, paint, semantics, and interaction. Rendering rebuilds a cheap description; Argui reconciles it against the retained tree.',
        ],
        code: { filename: 'src/view.rs', code: layout },
      },
      {
        id: 'keys',
        title: 'Use stable keys where identity matters',
        paragraphs: [
          'Interactive controls already require a stable key. Add keyed identities to siblings that can reorder, appear, or disappear so focus, animation, and retained state follow the logical item instead of its old position.',
        ],
        code: {
          filename: 'src/view.rs',
          code: 'Element::column(items.iter().map(|item| {\n    row(item).keyed(format!("item-{}", item.id))\n}))',
        },
      },
      {
        id: 'functions',
        title: 'Extract plain view functions',
        paragraphs: [
          'Start with ordinary functions returning Element. Extract a model only when a section needs state, lifecycle, subscriptions, tasks, or an independently cached presentation. This keeps the tree readable without creating a component abstraction for every wrapper.',
        ],
      },
    ],
  },
  {
    slug: 'start/counter',
    category: 'Start here',
    level: 'Beginner',
    minutes: 12,
    title: 'Build a counter with state',
    description:
      'Store application state in a model, handle a click, and invalidate only the presentation that changed.',
    example: example('counter', 'counter'),
    demoTitle: 'A compiled counter running in WebAssembly',
    sources: ['crates/argui-widget-gallery/src/pages/hot_reload.rs', 'docs/runtime/models.md'],
    sections: [
      {
        id: 'model',
        title: 'State lives in the model',
        paragraphs: [
          'The model is a normal Rust struct. Render receives &mut self and a Context tied to this presentation. There is no hidden global state or hook ordering.',
        ],
        code: { filename: 'src/counter.rs', code: counter },
      },
      {
        id: 'notify',
        title: 'Notify after a visible change',
        paragraphs: [
          'Mutating the struct is immediate. cx.notify() marks dependent presentations dirty and wakes the host. Multiple notifications in one transaction coalesce; an idle application does not continuously request frames.',
        ],
        note: 'If you forget cx.notify(), the value changes in memory but the view is not scheduled to rebuild.',
      },
      {
        id: 'events',
        title: 'Match stable targets',
        paragraphs: [
          'Events expose both stable keys and typed payloads. Match the control key, mutate state, then notify. Keyboard and accessibility activation synthesize the same click path as a pointer.',
        ],
      },
    ],
  },
  {
    slug: 'essentials/layout',
    category: 'Essentials',
    level: 'Beginner',
    minutes: 14,
    title: 'Responsive layout',
    description:
      'Use Flexbox, Grid, constraints, wrapping, and container queries without device-specific branches.',
    example: example('layout', 'layout'),
    demoTitle: 'Resize this compiled responsive layout',
    sources: [
      'crates/argui/examples/layout.rs',
      'docs/ui/styling.md',
      'crates/argui-layout/src/lib.rs',
    ],
    sections: [
      {
        id: 'automatic',
        title: 'Let constraints do the work',
        paragraphs: [
          'Rows, columns, wrapping, intrinsic text measurement, min/max size, and percentage values adapt automatically. Prefer these rules before reaching for a container query.',
        ],
        code: { filename: 'src/view.rs', code: layout },
      },
      {
        id: 'containers',
        title: 'Change presentation with a container query',
        paragraphs: [
          'A named container query is appropriate when narrow space requires a deliberate structural presentation change, such as turning a toolbar row into a column. Queries resolve against the nearest scope with the same identity.',
        ],
        code: {
          filename: 'src/view.rs',
          code: 'let scope = ContainerScopeId::new("toolbar");\nlet toolbar = Element::row(actions)\n    .when(ContainerQuery::max_width(scope, 360.0), compact_patch)\n    .container_scope(scope);',
        },
      },
      {
        id: 'mobile',
        title: 'Design for touch as well as width',
        paragraphs: [
          'Keep controls large enough to tap, allow action rows to wrap, and avoid hard-coded desktop widths. Hit slop can enlarge interaction geometry without changing pixels or layout.',
        ],
      },
    ],
  },
  {
    slug: 'essentials/events',
    category: 'Essentials',
    level: 'Beginner',
    minutes: 12,
    title: 'Events and interaction',
    description:
      'Handle pointer, keyboard, text, focus, and accessibility actions through one typed event system.',
    example: example('events', 'events'),
    demoTitle: 'Pointer, keyboard, and accessible activation',
    sources: [
      'docs/ui/interaction.md',
      'crates/argui/examples/interaction.rs',
      'crates/argui-ui/src/event.rs',
    ],
    sections: [
      {
        id: 'listeners',
        title: 'Attach typed listeners',
        paragraphs: [
          'Context::listener creates a handler owned by the current presentation. Element::on attaches it to any element. Dispatch follows capture, target, and bubbling phases.',
        ],
        code: { filename: 'src/view.rs', code: listener },
      },
      {
        id: 'payloads',
        title: 'Read the typed payload',
        paragraphs: [
          'UiEventKind distinguishes clicks, key input, text changes, gestures, scroll, dismiss, and selection changes. Check the kind before reading its payload and use target_key for stable application routing.',
        ],
      },
      {
        id: 'defaults',
        title: 'Preserve platform defaults deliberately',
        paragraphs: [
          'Use prevent_default only when the application replaces the normal behavior. stop_propagation ends bubbling; stop_immediate_propagation also stops later listeners on the current node. Passive listeners cannot prevent defaults.',
        ],
      },
    ],
  },
  {
    slug: 'essentials/styling',
    category: 'Essentials',
    level: 'Intermediate',
    minutes: 15,
    title: 'Style and theme',
    description:
      'Choose a color scheme and accent, override widget tokens, and apply typed visual state.',
    example: example('styling', 'styling'),
    demoTitle: 'An interactive theme configurator compiled to WebAssembly',
    sources: [
      'docs/ui/styling.md',
      'crates/argui-theme/src/lib.rs',
      'crates/argui-theme/src/tokens.rs',
      'crates/argui-widgets/src/theme.rs',
      'crates/argui-widget-gallery/src/app/theme.rs',
    ],
    sections: [
      {
        id: 'typed',
        title: 'Style with Rust values',
        paragraphs: [
          'Argui has no CSS parser or cascade. LayoutStyle, QuadStyle, TextStyle, and StylePatch make each property explicit and type checked.',
        ],
        code: {
          filename: 'src/view.rs',
          code: 'Element::column(content)\n    .padding(Sides::length(20.0))\n    .gap(12.0)\n    .background(theme.card)\n    .border(Border::all(1.0, theme.border))\n    .radius(CornerRadii::all(12.0))',
        },
      },
      {
        id: 'theme',
        title: 'Start from the default theme',
        paragraphs: [
          'default_theme(cx.environment()) builds matching light and dark WidgetTheme palettes. Resolve the active color scheme, then pass derived styles such as theme.button() or theme.input() to widgets.',
        ],
        code: {
          filename: 'src/view.rs',
          code: 'let themes = default_theme(cx.environment());\nlet theme = themes.resolve(cx.environment().color_scheme);\n\nButton::new("save", "Save", theme.button()).build()',
        },
      },
      {
        id: 'tokens',
        title: 'Override any token',
        paragraphs: [
          'ThemeOverrides is a sparse, typed map. Override only the colors or numeric effects your product owns; every other value continues to follow the default light or dark palette.',
        ],
        bullets: [
          'Surfaces: background, card, popover, muted, and borders',
          'Content: foreground, muted foreground, and destructive colors',
          'Controls: primary, secondary, focus ring, input, switch, and scrollbar values',
          'Effects: overlay blur, shadows, and dialog backdrop',
        ],
        note: 'The interactive example switches system, light, and dark palettes; changes the accent; then installs several token overrides together so their effect is visible immediately.',
      },
    ],
  },
  {
    slug: 'essentials/accessibility',
    category: 'Essentials',
    level: 'Intermediate',
    minutes: 14,
    title: 'Accessibility from the start',
    description:
      'Give controls stable semantics, keyboard behavior, focus, and useful touch targets.',
    example: example('accessibility', 'accessibility'),
    demoTitle: 'An accessible input in the browser semantic tree',
    sources: [
      'crates/argui/examples/accessibility.rs',
      'crates/argui-accessibility/src/schema.rs',
      'docs/ui/interaction.md',
    ],
    sections: [
      {
        id: 'widgets',
        title: 'Prefer semantic widgets',
        paragraphs: [
          'Built-in buttons, inputs, menus, dialogs, and collection widgets publish roles, names, values, states, and actions. Native targets use AccessKit; Web maintains real semantic HTML controls beside the canvas.',
        ],
      },
      {
        id: 'custom-semantics',
        title: 'Describe custom controls',
        paragraphs: [
          'Attach Semantics when a custom interactive element has no built-in widget. Give it a role, accessible name, current value or selection state, and the actions it accepts. Hide purely decorative descendants to avoid duplicate announcements.',
        ],
        code: {
          filename: 'src/view.rs',
          code: 'let semantics = Semantics::new(Role::Slider)\n    .label("Timeline zoom")\n    .value(format!("{:.0}%", zoom * 100.0));\ncontrol.semantics(semantics)',
        },
      },
      {
        id: 'focus',
        title: 'Manage focus as retained state',
        paragraphs: [
          'Focusable elements keep stable NodeIds across keyed reconciliation. Use focus scopes for restoring, trapped, or modal focus. Modal scopes also remove background nodes from the active semantic tree.',
        ],
        note: 'Test with keyboard-only input and a screen reader. A visually correct canvas does not prove that the semantic tree is correct.',
      },
    ],
  },
  {
    slug: 'essentials/animation',
    category: 'Essentials',
    level: 'Intermediate',
    minutes: 14,
    title: 'Animation and motion',
    description:
      'Compose transforms, layout morphs, color, vector transitions, and GPU aura effects in smooth loops.',
    example: example('animation', 'animation'),
    demoTitle: 'Six responsive motion patterns running together',
    sources: [
      'docs/ui/animation.md',
      'docs/rendering/effects.md',
      'crates/argui-widget-gallery/src/pages/motion.rs',
    ],
    sections: [
      {
        id: 'frame',
        title: 'Drive one coherent frame',
        paragraphs: [
          'A component opts into animation frames only while motion is active. One time value can coordinate translation, rotation, scale, layout, radii, Oklab color, opacity, and GPU layers without independent timers drifting apart.',
        ],
      },
      {
        id: 'practical',
        title: 'Compose practical motion',
        paragraphs: [
          'Use short loops to communicate status and longer one-shot transitions for state changes. The live lab combines a text aura, a layout morph, a vector crossfade, a sequenced loader, and a status pulse alongside a full composed transform.',
        ],
      },
      {
        id: 'responsive',
        title: 'Keep motion responsive and interruptible',
        paragraphs: [
          'The control bar remains visible while the cards scroll inside the Argui canvas. Cards wrap into one column on narrow viewports, scrolling stays available while every loop runs, and Pause stops frame requests immediately.',
        ],
        note: 'Reduced-motion preferences disable the loops and present a stable frame automatically.',
      },
    ],
  },
  {
    slug: 'advanced/tasks',
    category: 'Advanced',
    level: 'Intermediate',
    minutes: 16,
    title: 'Asynchronous tasks',
    description:
      'Run cancellable work, keep completion on the UI thread, and prevent stale results.',
    example: example('tasks', 'tasks'),
    demoTitle: 'Cancellable search over 10,000 rows',
    sources: ['crates/argui-widget-gallery/src/pages/async_tasks.rs', 'docs/runtime/tasks.md'],
    sections: [
      {
        id: 'feature',
        title: 'Enable tasks explicitly',
        paragraphs: [
          'The tasks feature uses Tokio on native targets and browser-local futures on WebAssembly. Applications without it do not pull in Tokio.',
        ],
        code: {
          filename: 'Cargo.toml',
          code: '[dependencies]\nargui = { version = "0.2.1", features = ["tasks", "widget-input", "widget-button", "widget-vlist"] }',
        },
      },
      {
        id: 'latest',
        title: 'Replace stale work',
        paragraphs: [
          'spawn_latest stores ownership in a TaskSlot. Starting a new search cancels the previous delivery, including an older result already queued for the UI thread.',
        ],
        code: { filename: 'src/search.rs', code: tasks },
      },
      {
        id: 'ownership',
        title: 'Choose the right owner',
        paragraphs: [
          'ModelContext::spawn binds work to model lifetime. Context::spawn additionally binds it to one presentation. Dropping the handle, closing its scope, unmounting its owner, or shutting down the runtime cancels delivery.',
        ],
      },
    ],
  },
  {
    slug: 'advanced/data',
    category: 'Advanced',
    level: 'Intermediate',
    minutes: 18,
    title: 'Lists, tables, and large data',
    description:
      'Render collections with stable identity and virtualize data that should not all be laid out at once.',
    example: example('data', 'data'),
    demoTitle: 'A sortable, selectable data table',
    sources: [
      'docs/widgets/lists-tables.md',
      'crates/argui-widget-gallery/src/pages/data_table.rs',
      'crates/argui-widget-gallery/src/pages/data.rs',
    ],
    sections: [
      {
        id: 'identity',
        title: 'Key rows by domain identity',
        paragraphs: [
          'A row key should come from the record ID, not its current array index. Stable keys preserve focus, selection, animation, and retained node identity when sorting or filtering.',
        ],
        code: {
          filename: 'src/table.rs',
          code: 'Element::column(rows.iter().map(|record| {\n    render_row(record).keyed(format!("record-{}", record.id))\n}))',
        },
      },
      {
        id: 'virtualize',
        title: 'Virtualize long collections',
        paragraphs: [
          'VList receives a stable key, row height, viewport height, and scroll offset. Its builder asks only for the visible range, so a 10,000-row model does not create 10,000 retained elements.',
        ],
      },
      {
        id: 'state',
        title: 'Keep selection in the model',
        paragraphs: [
          'Store sort order, selected record IDs, filters, and scroll state in the owning model. Derive visible rows during rendering or in a measured cache; do not duplicate the authoritative records inside view elements.',
        ],
      },
    ],
  },
  {
    slug: 'advanced/overlays',
    category: 'Advanced',
    level: 'Intermediate',
    minutes: 16,
    title: 'Dialogs, menus, and overlays',
    description:
      'Build focus-safe overlay surfaces and optionally host them outside the native window bounds.',
    example: example('overlays', 'overlays'),
    demoTitle: 'A modal dialog with focus management',
    sources: [
      'docs/widgets/overlays.md',
      'docs/platform/native-popovers.md',
      'crates/argui-widget-gallery/src/pages/popover.rs',
    ],
    sections: [
      {
        id: 'retained',
        title: 'Mount overlays as retained UI',
        paragraphs: [
          'Dialogs, popovers, tooltips, menus, sheets, and drawers keep their open state in the model. Their elements participate in normal layout, paint, event dispatch, and semantics.',
        ],
      },
      {
        id: 'focus',
        title: 'Select the correct focus scope',
        paragraphs: [
          'A modal scope traps Tab navigation and removes background semantics. A restoring scope remembers the previous focus target. Dismiss events provide one path for Escape, outside press, and host requests.',
        ],
      },
      {
        id: 'native',
        title: 'Cross window bounds only when needed',
        paragraphs: [
          'The native-popups feature creates platform-hosted surfaces for menus and popovers that must extend outside the parent window. Browser builds and unsupported backends retain an in-window fallback.',
        ],
        note: 'Native popup support is opt-in because it adds platform-specific hosting and lifecycle behavior.',
      },
    ],
  },
  {
    slug: 'advanced/i18n',
    category: 'Advanced',
    level: 'Intermediate',
    minutes: 15,
    title: 'Localization and RTL',
    description:
      'Embed Fluent catalogs, negotiate locales, format plurals, and keep layout direction consistent.',
    example: example('i18n', 'i18n'),
    demoTitle: 'English, French, Arabic, plurals, and RTL',
    sources: [
      'docs/i18n.md',
      'crates/argui-i18n/src/lib.rs',
      'crates/argui-widget-gallery/src/pages/i18n.rs',
    ],
    sections: [
      {
        id: 'catalogs',
        title: 'Embed catalogs in every target',
        paragraphs: [
          'include_str! keeps the same resources available on desktop and WebAssembly. Catalog construction reports invalid Fluent syntax and duplicate identifiers before rendering begins.',
        ],
        code: {
          filename: 'src/i18n.rs',
          code: 'let english = Catalog::parse(\n    langid!("en-US"),\n    include_str!("locales/en-US/main.ftl"),\n)?;\nlet french = Catalog::parse(\n    langid!("fr"),\n    include_str!("locales/fr/main.ftl"),\n)?;',
        },
      },
      {
        id: 'format',
        title: 'Pass typed variables',
        paragraphs: [
          'Use text for messages without variables and format with FluentArgs for names, numbers, and plurals. Missing messages, attributes, and invalid formatting return distinct errors.',
        ],
      },
      {
        id: 'direction',
        title: 'Apply writing direction to the tree',
        paragraphs: [
          'Use Localizer::is_rtl for the direction scope and for collection widgets whose horizontal arrow behavior changes in RTL. This keeps copy, layout, overlays, and keyboard behavior aligned.',
        ],
      },
    ],
  },
  {
    slug: 'architecture/mental-model',
    category: 'Architecture',
    level: 'Intermediate',
    minutes: 18,
    title: 'The Argui mental model',
    description:
      'Understand models, presentations, retained elements, layout, paint, semantics, and host boundaries.',
    example: example('mental-model', 'mental_model'),
    demoTitle: 'The retained pipeline in a running application',
    sources: ['docs/architecture.md', 'docs/runtime/models.md', 'docs/rendering/primitives.md'],
    sections: [
      {
        id: 'pipeline',
        title: 'Data flows through explicit layers',
        paragraphs: [
          'A model renders an Element description. The retained UI reconciles stable nodes, Taffy computes logical geometry, Cosmic Text shapes glyphs, paint records renderer-neutral primitives, and WGPU submits the final frame. Accessibility consumes a parallel semantic tree.',
        ],
        code: {
          filename: 'Pipeline',
          code: 'model → Element tree → retained update\n                    ├→ layout → shaped text → paint → WGPU\n                    └→ semantics → AccessKit / browser DOM',
        },
      },
      {
        id: 'updates',
        title: 'Not every change costs a layout',
        paragraphs: [
          'UiTree classifies updates as none, semantics, paint, scroll, or layout. A color change can reuse geometry and shaped text; a semantic-only change can avoid GPU submission; an unchanged rebuild performs no presentation work.',
        ],
      },
      {
        id: 'presentations',
        title: 'Separate model data from presentations',
        paragraphs: [
          'Entity retains model data and model-owned resources. Mount represents one presentation with its own environment, cache, handlers, dependencies, and view tasks. One model may therefore appear in multiple windows without merging their presentation state.',
        ],
      },
    ],
  },
  {
    slug: 'architecture/project-structure',
    category: 'Architecture',
    level: 'Intermediate',
    minutes: 13,
    title: 'Structure a real application',
    description:
      'Organize models, views, services, and platform edges without coupling your product to the renderer.',
    example: example('project-structure', 'project_structure'),
    demoTitle: 'A model, task, view, and virtual list working together',
    sources: [
      'app_examples/fake-ai-harness/src/app.rs',
      'app_examples/fake-ai-harness/src/app/view.rs',
      'docs/repo/structure.md',
    ],
    sections: [
      {
        id: 'folders',
        title: 'Split by responsibility',
        paragraphs: [
          'Keep the application model, its view composition, domain services, and platform adapters distinct. A practical starting point is app.rs for state and event routing, app/view.rs for Element construction, and focused service modules for external work.',
        ],
        code: {
          filename: 'Suggested structure',
          code: 'src/\n  main.rs          # target entry point\n  app.rs           # state and event routing\n  app/view.rs      # Element composition\n  services/        # domain I/O\n  theme.rs         # product tokens',
        },
      },
      {
        id: 'boundaries',
        title: 'Keep boundaries renderer-neutral',
        paragraphs: [
          'Domain services should return ordinary Rust values. The model decides when to launch work and how results change state. The view consumes state and returns Elements; it should not own network clients or platform windows.',
        ],
      },
      {
        id: 'models',
        title: 'Create child models for ownership',
        paragraphs: [
          'Use a child Entity when a feature needs independent state, tasks, subscriptions, lifecycle, or cached rendering. Use a plain view function when it only transforms inputs into elements.',
        ],
      },
    ],
  },
  {
    slug: 'architecture/clean-code',
    category: 'Architecture',
    level: 'Advanced',
    minutes: 16,
    title: 'Write clean Argui code',
    description:
      'Keep state authoritative, views readable, updates bounded, and feature costs visible.',
    example: example('clean-code', 'clean_code'),
    demoTitle: 'A larger feature split into focused state and view code',
    sources: [
      'docs/contributing/code-quality.md',
      'crates/argui-widget-gallery/src/pages/data_table.rs',
      'app_examples/fake-ai-harness/src/app/view.rs',
    ],
    sections: [
      {
        id: 'rules',
        title: 'Prefer explicit data flow',
        paragraphs: [
          'Use plain structs and enums, short event routes, and view functions named after product concepts. Keep one authoritative value for each piece of state and derive presentation values during render.',
        ],
        bullets: [
          'Stable domain keys for interactive or reorderable elements',
          'No I/O or unbounded allocation in render',
          'No speculative abstractions or generic utils modules',
          'Feature flags only for integrations the application actually uses',
        ],
      },
      {
        id: 'updates',
        title: 'Invalidate with intent',
        paragraphs: [
          'Call notify after state changes that affect a presentation. Keep paint-only changes as paint properties and avoid rebuilding ownership graphs for temporary visual states. Measure before introducing caches.',
        ],
      },
      {
        id: 'testing',
        title: 'Test behavior at the right boundary',
        paragraphs: [
          'Test state transitions, invalidation class, layout results, semantics, and event translation through public behavior. Browser checks must verify both pixels and semantic controls; native GUI checks run on a private display.',
        ],
        note: 'Argui itself enforces formatting, Clippy, WebAssembly checks, source hygiene, and an 85% minimum for lines, functions, regions, and branches.',
      },
    ],
  },
  {
    slug: 'architecture/custom-elements',
    category: 'Architecture',
    level: 'Advanced',
    minutes: 22,
    title: 'Create a custom element',
    description:
      'Own custom measurement and paint while keeping children, interaction, semantics, and invalidation in Argui.',
    example: example('custom-elements', 'custom_elements'),
    demoTitle: 'A draggable custom timeline rendered in WebAssembly',
    sources: [
      'docs/ui/custom-elements.md',
      'crates/argui-widget-gallery/src/pages/timeline.rs',
      'crates/argui-widget-gallery/src/pages/timeline/regions.rs',
    ],
    sections: [
      {
        id: 'when',
        title: 'Use a custom element for a real boundary',
        paragraphs: [
          'CustomElement is for geometry or paint that standard rows, columns, Grid, widgets, and effects cannot express cleanly. The element can measure itself, place retained children, prepare cached paint state, and record paint primitives.',
        ],
      },
      {
        id: 'contract',
        title: 'Implement the retained contract',
        paragraphs: [
          'State persists beside the retained node. layout_revision changes when measurement or child placement changes. paint_revision changes when prepared or recorded pixels change. Accurate revisions let Argui skip work safely.',
        ],
        code: { filename: 'src/ruler.rs', code: custom },
      },
      {
        id: 'integrate',
        title: 'Keep behavior and semantics outside paint',
        paragraphs: [
          'Attach gestures and listeners through normal Element interaction, and attach Semantics for assistive technology. Paint does not implicitly define hit geometry. Use a stable key and explicit HitTestStyle when the custom shape needs non-rectangular or enlarged targets.',
        ],
        note: 'The complete timeline source places retained button children, caches ruler ticks, supports pan and keyboard edits, and shares clip data across two mounted views.',
      },
    ],
  },
]

export const docCategories = ['Start here', 'Essentials', 'Advanced', 'Architecture'] as const
export const docsByCategory = docCategories.map((category) => ({
  category,
  guides: docs.filter((guide) => guide.category === category),
}))
export const findDoc = (slug: string) => docs.find((guide) => guide.slug === slug)
