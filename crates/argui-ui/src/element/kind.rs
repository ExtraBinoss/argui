use argui_paint::{Color, ImageFit, ImageId, ImageSampling, VectorId};
use argui_text::{TextContent, TextStyle};

impl std::fmt::Debug for super::Element {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.text_privacy.protected() {
            formatter
                .debug_struct("Element")
                .field("key", &self.key)
                .field("content", &"[protected]")
                .finish()
        } else {
            std::fmt::Debug::fmt(&self.0, formatter)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElementNode {
    pub inspectable: bool,
    /// Isolates content sizing; see [`super::Element::layout_boundary`] for the contract.
    pub layout_boundary: bool,
    pub(crate) retained_identity: Option<crate::RetainedIdentity>,
    pub key: Option<String>,
    pub kind: ElementKind,
    pub native_content: Option<crate::NativeContent>,
    pub desktop_backdrop: Option<crate::DesktopBackdrop>,
    pub style: crate::LayoutStyle,
    /// Inherited layout direction, with nested scopes overriding their ancestors.
    pub direction_scope: Option<crate::WritingDirection>,
    pub paint: argui_paint::PaintStyle,
    pub transform: argui_core::Transform2D,
    pub transform_origin: argui_core::TransformOrigin,
    pub interaction: Option<crate::Interaction>,
    pub hit_test: crate::HitTestStyle,
    pub(crate) conditional_styles: crate::state::ConditionalStyles,
    pub(crate) style_transition: Option<crate::StyleTransition>,
    pub(crate) state_scope: Option<crate::StateScopeId>,
    pub(crate) container_scope: Option<crate::ContainerScopeId>,
    pub(crate) active_states: Vec<crate::StateName>,
    pub semantics: Option<Box<argui_accessibility::Semantics>>,
    /// Optional hover/focus help, presented by an application tooltip host.
    pub tooltip: Option<String>,
    pub semantic_hidden: bool,
    pub semantic_scope: bool,
    pub semantic_bindings: crate::SemanticBindings,
    pub bindings: Vec<crate::PropertyBinding>,
    pub layer: Option<Box<argui_paint::LayerStyle>>,
    pub effects: Vec<crate::ScopedEffect>,
    pub scroll: Option<Box<crate::ScrollConfig>>,
    pub(crate) virtual_item: Option<crate::VirtualItem>,
    pub portal: Option<crate::Portal>,
    pub focus_scope: Option<crate::FocusScope>,
    pub text_privacy: crate::TextPrivacy,
    pub text_history: Option<crate::HistoryConfig>,
    pub action_scope: Option<crate::ActionScope>,
    pub action: Option<crate::ActionInvocation>,
    pub event_listeners: Vec<crate::EventListener>,
    pub user_select: crate::UserSelect,
    pub selection_style: Option<crate::TextSelectionStyle>,
    pub selection_highlight: Option<crate::TextSelectionHighlight>,
    pub z_index: i32,
    pub children: Vec<super::Element>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    Custom(crate::CustomDescription),
    GpuCanvas(crate::GpuCanvasSpec),
    Container,
    Text {
        content: TextContent,
        style: TextStyle,
    },
    TextEditor {
        value: String,
        placeholder: String,
        styled: Option<Box<TextContent>>,
        multiline: bool,
        read_only: bool,
        filter: crate::TextInputFilter,
        text: TextStyle,
        placeholder_text: Box<TextStyle>,
        selection: Color,
        caret: crate::CaretStyle,
    },
    Image {
        image: ImageId,
        fit: ImageFit,
        sampling: ImageSampling,
    },
    Vector {
        vector: VectorId,
        fit: ImageFit,
        color: Color,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextEditorSpec {
    pub value: String,
    pub placeholder: String,
    pub multiline: bool,
    pub read_only: bool,
    pub filter: crate::TextInputFilter,
    pub text: TextStyle,
    pub placeholder_text: TextStyle,
    pub selection: Color,
    pub caret: crate::CaretStyle,
}
