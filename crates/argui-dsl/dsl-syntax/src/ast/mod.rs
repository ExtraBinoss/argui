use crate::{SyntaxKind, SyntaxNode, SyntaxToken};

/// Common casting and syntax access implemented by typed AST facade nodes.
pub trait AstNode: Sized {
    /// Returns whether `kind` can represent this AST node.
    fn can_cast(kind: SyntaxKind) -> bool;
    /// Casts a syntax node when its kind matches this AST type.
    fn cast(syntax: SyntaxNode) -> Option<Self>;
    /// Returns the underlying lossless syntax node.
    fn syntax(&self) -> &SyntaxNode;
}

macro_rules! ast_node {
    ($name:ident, $kind:ident) => {
        #[doc = concat!("Typed AST facade for `", stringify!($kind), "` syntax.")]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$kind
            }

            fn cast(syntax: SyntaxNode) -> Option<Self> {
                Self::can_cast(syntax.kind()).then_some(Self(syntax))
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

ast_node!(SourceFile, Root);
ast_node!(ImportDecl, ImportDecl);
ast_node!(StructDecl, StructDecl);
ast_node!(EnumDecl, EnumDecl);
ast_node!(ComponentDecl, ComponentDecl);
ast_node!(ThemeDecl, ThemeDecl);
ast_node!(StyleDecl, StyleDecl);
ast_node!(EffectDecl, EffectDecl);
ast_node!(FunctionDecl, FunctionDecl);
ast_node!(PropertyDecl, PropertyDecl);
ast_node!(CallbackDecl, CallbackDecl);
ast_node!(SlotDecl, SlotDecl);
ast_node!(Element, Element);
ast_node!(PropertyAssignment, PropertyAssignment);
ast_node!(Expression, Expr);

/// Top-level declaration exposed through the typed AST facade.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Declaration {
    Import(ImportDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Component(ComponentDecl),
    Theme(ThemeDecl),
    Style(StyleDecl),
    Effect(EffectDecl),
    Function(FunctionDecl),
}

impl SourceFile {
    /// Iterates over all successfully recognized top-level declarations.
    pub fn declarations(&self) -> impl Iterator<Item = Declaration> + '_ {
        self.syntax().children().filter_map(Declaration::cast)
    }
}

impl Declaration {
    /// Casts any supported top-level declaration node.
    ///
    /// * `node` — candidate lossless syntax node.
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::ImportDecl => ImportDecl::cast(node).map(Self::Import),
            SyntaxKind::StructDecl => StructDecl::cast(node).map(Self::Struct),
            SyntaxKind::EnumDecl => EnumDecl::cast(node).map(Self::Enum),
            SyntaxKind::ComponentDecl => ComponentDecl::cast(node).map(Self::Component),
            SyntaxKind::ThemeDecl => ThemeDecl::cast(node).map(Self::Theme),
            SyntaxKind::StyleDecl => StyleDecl::cast(node).map(Self::Style),
            SyntaxKind::EffectDecl => EffectDecl::cast(node).map(Self::Effect),
            SyntaxKind::FunctionDecl => FunctionDecl::cast(node).map(Self::Function),
            _ => None,
        }
    }

    /// Returns the underlying declaration syntax.
    #[must_use]
    pub fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Import(node) => node.syntax(),
            Self::Struct(node) => node.syntax(),
            Self::Enum(node) => node.syntax(),
            Self::Component(node) => node.syntax(),
            Self::Theme(node) => node.syntax(),
            Self::Style(node) => node.syntax(),
            Self::Effect(node) => node.syntax(),
            Self::Function(node) => node.syntax(),
        }
    }
}

/// Returns the first identifier token directly owned by `node`.
#[must_use]
pub fn name_token(node: &impl AstNode) -> Option<SyntaxToken> {
    node.syntax()
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
}

impl ComponentDecl {
    /// Returns the component's declared name token.
    #[must_use]
    pub fn name(&self) -> Option<SyntaxToken> {
        name_token(self)
    }

    /// Iterates over declared component properties.
    pub fn properties(&self) -> impl Iterator<Item = PropertyDecl> + '_ {
        self.syntax().descendants().filter_map(PropertyDecl::cast)
    }

    /// Iterates over visual element sites in source order.
    pub fn elements(&self) -> impl Iterator<Item = Element> + '_ {
        self.syntax().descendants().filter_map(Element::cast)
    }
}
