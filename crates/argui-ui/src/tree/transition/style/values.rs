use argui_paint::{BorderWidths, CornerRadii};

pub(super) const fn widths_from_array(value: [f32; 4]) -> BorderWidths {
    BorderWidths {
        left: value[0],
        right: value[1],
        top: value[2],
        bottom: value[3],
    }
}

pub(super) const fn radii(value: [f32; 4]) -> CornerRadii {
    CornerRadii {
        top_left: value[0],
        top_right: value[1],
        bottom_right: value[2],
        bottom_left: value[3],
    }
}
