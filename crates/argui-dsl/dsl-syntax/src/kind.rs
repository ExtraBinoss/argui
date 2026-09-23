macro_rules! define_syntax_kinds {
    ($($kind:ident),+ $(,)?) => {
        /// Token and node kinds used by the lossless Argui syntax tree.
        #[repr(u16)]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum SyntaxKind {
            $($kind),+
        }

        impl SyntaxKind {
            pub(crate) const ALL: &'static [Self] = &[$(Self::$kind),+];
        }
    };
}

define_syntax_kinds!(
    Eof,
    ErrorToken,
    Whitespace,
    LineComment,
    BlockComment,
    Ident,
    Number,
    String,
    ThemeName,
    ImportKw,
    FromKw,
    AsKw,
    ExportKw,
    StructKw,
    EnumKw,
    ComponentKw,
    ThemeKw,
    StyleKw,
    EffectKw,
    FnKw,
    ForKw,
    InKw,
    KeyKw,
    IfKw,
    ElseKw,
    PropertyKw,
    OutKw,
    InOutKw,
    PrivateKw,
    CallbackKw,
    SlotKw,
    OnKw,
    StatesKw,
    WhenKw,
    AnimateKw,
    ParameterKw,
    ShaderKw,
    TrueKw,
    FalseKw,
    NullKw,
    ForTargetKw,
    LetKw,
    ReturnKw,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Hash,
    At,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Eq,
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Arrow,
    FatArrow,
    TwoWay,
    Question,
    Root,
    Error,
    ImportDecl,
    ImportList,
    ImportItem,
    StructDecl,
    EnumDecl,
    ComponentDecl,
    ThemeDecl,
    StyleDecl,
    EffectDecl,
    FunctionDecl,
    FunctionParameter,
    FieldDecl,
    VariantDecl,
    PropertyDecl,
    CallbackDecl,
    CallbackParameter,
    SlotDecl,
    SlotParameter,
    SlotContent,
    EffectParameterDecl,
    EffectApplication,
    Element,
    ElementId,
    PropertyAssignment,
    TwoWayBinding,
    EventBlock,
    Statement,
    IfStatement,
    Block,
    ForExpr,
    IfExpr,
    ElseBranch,
    StatesBlock,
    StateDecl,
    AnimateDecl,
    KeyframesDecl,
    KeyframeDecl,
    ThemeTokenDecl,
    ThemeModeDecl,
    StyleStateDecl,
    StyleApplication,
    TypeRef,
    Expr,
    LiteralExpr,
    PathExpr,
    CallExpr,
    MemberExpr,
    UnaryExpr,
    BinaryExpr,
    ConditionalExpr,
    ArrayExpr,
    IndexExpr,
    StructExpr,
    StructFieldExpr,
    ArgumentList,
    Missing,
);

impl SyntaxKind {
    /// Returns whether this token contains formatting trivia rather than syntax.
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::Whitespace | Self::LineComment | Self::BlockComment
        )
    }

    /// Returns whether this kind can start a top-level declaration.
    #[must_use]
    pub const fn is_declaration_start(self) -> bool {
        matches!(
            self,
            Self::ImportKw
                | Self::ExportKw
                | Self::StructKw
                | Self::EnumKw
                | Self::ComponentKw
                | Self::ThemeKw
                | Self::StyleKw
                | Self::EffectKw
                | Self::FnKw
        )
    }
}
