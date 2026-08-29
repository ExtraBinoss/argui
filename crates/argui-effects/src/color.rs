use argui_paint::Filter;

macro_rules! color_preset {
    ($name:ident, $variant:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(pub f32);

        impl $name {
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
