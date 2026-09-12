use argui_paint::ClipChain;
use argui_ui::NodeId;

/// Visible shape in the UI tree's logical coordinates, including rounded ancestor clips.
#[derive(Clone, Debug, PartialEq)]
pub struct DesktopBackdropRegion {
    pub node: NodeId,
    pub shape: ClipChain,
}
