use argui_paint::Filter;

macro_rules! color_preset {
    ($name:ident, $variant:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        /// Scalar color adjustment preset; the meaning of the value follows its type.
        pub struct $name(pub f32);

        impl $name {
            /// Converts this preset into the corresponding paint filter.
            #[must_use]
            pub const fn filter(self) -> Filter {
                Filter::$variant(self.0)
            }
        }
    };
}

color_preset!(Brightness, Brightness);
color_preset!(Contrast, Contrast);
color_preset!(Saturation, Saturation);
color_preset!(HueRotate, HueRotate);
color_preset!(Opacity, Opacity);
