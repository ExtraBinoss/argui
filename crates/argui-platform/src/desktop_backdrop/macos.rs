use super::{BackdropBackend, BackdropError, Request};
use argui_core::{BackdropMaterial, ColorScheme, Rect};
use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSAutoresizingMaskOptions, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindow,
};
use objc2_core_graphics::CGMutablePath;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::CAShapeLayer;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub(super) struct Backend {
    window: Retained<NSWindow>,
    content: Retained<NSView>,
    container: Retained<NSView>,
    effect: Retained<NSVisualEffectView>,
    mask: Retained<CAShapeLayer>,
    autoresizing: NSAutoresizingMaskOptions,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl BackdropBackend for Backend {
    #[allow(unsafe_code)]
    fn new(_: RawDisplayHandle, handle: RawWindowHandle) -> Result<Self, BackdropError> {
        let RawWindowHandle::AppKit(handle) = handle else {
            return Err(BackdropError::Unsupported);
        };
        let mtm = MainThreadMarker::new().ok_or(BackdropError::Unsupported)?;
        // SAFETY: the live NSView is retained by NativeBackdrop's owner. All AppKit work is
        // on the main thread. Retaining the view and window also keeps the wrapper's children alive.
        let content = unsafe { Retained::retain(handle.ns_view.cast::<NSView>().as_ptr()) }
            .ok_or(BackdropError::Unsupported)?;
        let window = content.window().ok_or(BackdropError::Unsupported)?;
        if window.contentView().as_deref() != Some(&content) {
            return Err(BackdropError::Unsupported);
        }
        let container = NSView::initWithFrame(NSView::alloc(mtm), content.frame());
        let effect =
            NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), content.bounds());
        effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        effect.setState(NSVisualEffectState::FollowsWindowActiveState);
        effect.setWantsLayer(true);
        effect.setHidden(true);
        let mask = CAShapeLayer::new();
        // SAFETY: this layer is dedicated to this effect; it cannot form a layer cycle.
        unsafe {
            effect
                .layer()
                .ok_or(BackdropError::Unsupported)?
                .setMask(Some(&mask));
        }
        let autoresizing = content.autoresizingMask();
        let flexible = NSAutoresizingMaskOptions::ViewWidthSizable
            | NSAutoresizingMaskOptions::ViewHeightSizable;
        container.setAutoresizingMask(flexible);
        content.setAutoresizingMask(flexible);
        effect.setAutoresizingMask(flexible);
        // The GPU view must remain above the native material. A child of the GPU view
        // would instead cover rendered text and intercept input.
        window.setContentView(Some(&container));
        container.addSubview(&effect);
        container.addSubview(&content);
        content.setFrame(container.bounds());
        Ok(Self {
            window,
            content,
            container,
            effect,
            mask,
            autoresizing,
        })
    }
    fn available(&mut self) -> Result<bool, BackdropError> {
        Ok(true)
    }
    #[allow(unsafe_code)]
    fn apply(&mut self, regions: &[Rect], request: &Request) -> Result<(), BackdropError> {
        let size = NSSize::new(
            f64::from(request.size.width),
            f64::from(request.size.height),
        );
        self.effect
            .setFrame(NSRect::new(NSPoint::new(0.0, 0.0), size));
        self.mask
            .setFrame(NSRect::new(NSPoint::new(0.0, 0.0), size));
        self.effect.setMaterial(match request.material {
            BackdropMaterial::Glass => NSVisualEffectMaterial::Popover,
            BackdropMaterial::Sidebar => NSVisualEffectMaterial::Sidebar,
            BackdropMaterial::Header => NSVisualEffectMaterial::HeaderView,
        });
        // SAFETY: these are AppKit's process-lifetime appearance constants; null path
        // transforms are explicitly permitted by CGPathAddRect. Geometry is validated upstream.
        unsafe {
            let name = if request.scheme == ColorScheme::Dark {
                NSAppearanceNameDarkAqua
            } else {
                NSAppearanceNameAqua
            };
            self.effect
                .setAppearance(NSAppearance::appearanceNamed(name).as_deref());
            let path = CGMutablePath::new();
            for rect in regions {
                let rect = NSRect::new(
                    NSPoint::new(
                        f64::from(rect.origin.x),
                        f64::from(request.size.height - rect.origin.y - rect.size.height),
                    ),
                    NSSize::new(f64::from(rect.size.width), f64::from(rect.size.height)),
                );
                CGMutablePath::add_rect(Some(&path), std::ptr::null(), rect);
            }
            self.mask.setPath(Some(&path));
        }
        self.effect.setHidden(regions.is_empty());
        Ok(())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for Backend {
    fn drop(&mut self) {
        self.effect.removeFromSuperview();
        if self.window.contentView().as_deref() == Some(&self.container) {
            self.content.removeFromSuperview();
            self.content.setAutoresizingMask(self.autoresizing);
            self.window.setContentView(Some(&self.content));
        }
    }
}
