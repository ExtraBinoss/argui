use crate::fake_model::FakeStream;
use argui::{
    animation::Frame,
    runtime::{Context, Entity, LayoutSnapshot, Render},
    ui::{Element, EventType, UiEventKind, VirtualList, VirtualWindow},
    widgets::{MessageScrollState, TablerIcon, WidgetAssets, shadcn},
};
use std::{collections::VecDeque, time::Duration};
use web_time::Instant;

mod view;

const TARGET_RATE: u32 = 1_000;
const TARGET_TOKENS: usize = 6_000;
const TOKENS_PER_MESSAGE: usize = 128;
const MESSAGE_ESTIMATE: f32 = 176.0;
const FADE_TIME: Duration = Duration::from_millis(150);
const GRAPH_SAMPLES: usize = 32;

#[derive(Debug)]
struct StreamChunk {
    key: String,
    text: String,
    tokens: usize,
    source_label: bool,
    born: Instant,
    fade_complete: bool,
    reduced_motion: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Phase {
    #[default]
    Ready,
    Streaming,
    Complete,
}

pub struct AiHarness {
    prompt: String,
    submitted_prompt: String,
    chunks: Vec<Entity<StreamChunk>>,
    stream: Option<FakeStream>,
    phase: Phase,
    started: Option<Instant>,
    elapsed: Duration,
    completed_at: Option<Duration>,
    first_token: Option<Duration>,
    batches: u64,
    last_batch: usize,
    recent_batches: VecDeque<usize>,
    message_scroll: MessageScrollState,
    message_heights: VirtualList,
    message_window: Option<VirtualWindow>,
    conversation_offset: f32,
    conversation_viewport: f32,
    scroll_maximum: f32,
    pending_appends: usize,
    compact: bool,
    assets: WidgetAssets,
}

impl Default for AiHarness {
    fn default() -> Self {
        Self {
            prompt: "What is an LLM?".into(),
            submitted_prompt: String::new(),
            chunks: Vec::new(),
            stream: None,
            phase: Phase::Ready,
            started: None,
            elapsed: Duration::ZERO,
            completed_at: None,
            first_token: None,
            batches: 0,
            last_batch: 0,
            recent_batches: VecDeque::with_capacity(GRAPH_SAMPLES),
            message_scroll: MessageScrollState::default(),
            message_heights: VirtualList::variable(0, MESSAGE_ESTIMATE, 280.0).overscan(10),
            message_window: None,
            conversation_offset: 0.0,
            conversation_viewport: 280.0,
            scroll_maximum: 0.0,
            pending_appends: 0,
            compact: false,
            assets: WidgetAssets::tabler_subset(argui::core::Color::WHITE, [TablerIcon::ArrowDown]),
        }
    }
}

impl AiHarness {
    fn start(&mut self, cx: &mut Context<Self>) {
        let prompt = self.prompt.trim();
        if prompt.is_empty() {
            return;
        }
        self.submitted_prompt = prompt.into();
        self.chunks.clear();
        self.message_heights =
            VirtualList::variable(1, MESSAGE_ESTIMATE, self.conversation_viewport).overscan(10);
        self.message_window = None;
        self.conversation_offset = 0.0;
        self.stream = Some(FakeStream::new(TARGET_RATE, TARGET_TOKENS, prompt));
        self.phase = Phase::Streaming;
        self.started = Some(Instant::now());
        self.elapsed = Duration::ZERO;
        self.completed_at = None;
        self.first_token = None;
        self.batches = 0;
        self.last_batch = 0;
        self.recent_batches.clear();
        self.message_scroll = MessageScrollState::default();
        self.scroll_maximum = 0.0;
        self.pending_appends = 1;
        cx.notify();
    }

    fn advance(&mut self, cx: &mut Context<Self>) {
        let Some(started) = self.started else {
            return;
        };
        self.elapsed = started.elapsed();
        let mut changed = false;
        if self.phase == Phase::Streaming {
            let Some(stream) = self.stream.as_mut() else {
                return;
            };
            let batch = stream.take_due(self.elapsed);
            let finished = stream.is_finished();
            if batch.tokens > 0 {
                self.first_token.get_or_insert(self.elapsed);
                self.push_batch(batch.text, batch.tokens, cx);
                self.last_batch = batch.tokens;
                self.batches += 1;
                if self.recent_batches.len() == GRAPH_SAMPLES {
                    self.recent_batches.pop_front();
                }
                self.recent_batches.push_back(batch.tokens);
                changed = true;
            }
            if finished {
                self.phase = Phase::Complete;
                self.completed_at = Some(self.elapsed);
                changed = true;
            }
        }
        if changed {
            cx.notify();
        }
    }

    fn push_batch(&mut self, text: String, tokens: usize, cx: &mut Context<Self>) {
        let active = self
            .chunks
            .last()
            .filter(|chunk| chunk.read(|chunk| chunk.tokens < TOKENS_PER_MESSAGE))
            .cloned();
        if let Some(active) = active {
            active.update(|chunk, cx| {
                chunk.text.push_str(&text);
                chunk.tokens = chunk.tokens.saturating_add(tokens);
                cx.notify();
            });
            return;
        }
        let chunk = cx.new_entity(StreamChunk {
            key: format!("message-agent-{}", self.chunks.len() + 1),
            text,
            tokens,
            source_label: self.chunks.is_empty(),
            born: Instant::now(),
            fade_complete: false,
            reduced_motion: false,
        });
        self.chunks.push(chunk);
        self.message_heights
            .insert(self.message_heights.item_count(), 1);
        self.pending_appends = self.pending_appends.saturating_add(1);
    }

    fn emitted(&self) -> usize {
        self.stream.as_ref().map_or(0, FakeStream::emitted)
    }

    fn observed_rate(&self) -> f64 {
        let seconds = self.completed_at.unwrap_or(self.elapsed).as_secs_f64();
        if seconds > 0.0 {
            self.emitted() as f64 / seconds
        } else {
            0.0
        }
    }

    fn handle_scroll(&mut self, event: &argui::ui::UiEvent, cx: &mut Context<Self>) {
        let mut window_changed = false;
        if event.target_key() == Some("conversation")
            && let UiEventKind::Scrolled { offset, .. } = event.kind
        {
            self.conversation_offset = offset.y.max(0.0);
            window_changed = self.message_window.as_ref()
                != Some(&self.message_heights.window(self.conversation_offset));
        }
        let previous = self.message_scroll.clone();
        self.message_scroll
            .observe(event, "conversation", self.scroll_maximum);
        if window_changed || self.message_scroll != previous {
            cx.notify();
        }
    }
}

impl Render for AiHarness {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.message_window = Some(self.message_heights.window(self.conversation_offset));
        let environment = cx.environment();
        let themes = shadcn(&environment);
        let theme = themes.resolve(environment.color_scheme);
        self.view(cx, theme)
            .safe_area(environment.safe_area_insets)
            .on(cx.listener(EventType::Scroll, Self::handle_scroll))
    }

    fn wants_animation_frame(&self) -> bool {
        self.phase == Phase::Streaming
    }

    fn animation_frame(&mut self, _frame: Frame, cx: &mut Context<Self>) {
        self.advance(cx);
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        let compact = layout.viewport_size().width < 760.0;
        let viewport = if compact {
            (layout.viewport_size().height - 250.0).clamp(180.0, 560.0)
        } else {
            (layout.viewport_size().height - 280.0).clamp(220.0, 600.0)
        };
        if self.compact != compact || (self.conversation_viewport - viewport).abs() > 0.5 {
            self.compact = compact;
            self.conversation_viewport = viewport;
            self.message_heights = self
                .message_heights
                .clone()
                .with_viewport(viewport)
                .overscan(10);
            cx.notify();
        }
        let maximum = (self.message_heights.total_extent() - self.conversation_viewport).max(0.0);
        let maximum_changed = (self.scroll_maximum - maximum).abs() > 0.5;
        self.scroll_maximum = maximum;
        let appends = std::mem::take(&mut self.pending_appends);
        let unread = self.message_scroll.unread;
        if (appends > 0 || maximum_changed && self.message_scroll.following)
            && let Some(request) =
                self.message_scroll
                    .appended("conversation", appends, self.scroll_maximum)
        {
            self.conversation_offset = self.scroll_maximum;
            cx.scroll(request);
        }
        if self.message_scroll.unread != unread {
            cx.notify();
        }
    }

    fn vector_assets(&self) -> Vec<argui::paint::VectorAsset> {
        self.assets.assets().to_vec()
    }
}
