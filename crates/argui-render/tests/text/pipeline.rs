#![cfg(not(target_arch = "wasm32"))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Real glyph-pipeline captures using fixed fonts and offscreen sRGB targets.

use std::path::{Path, PathBuf};

use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{ClipChain, DisplayList};
use argui_render::RendererError;
use argui_text::{FontFamily, TextBlock, TextEngine, TextScene};

#[allow(dead_code)]
#[path = "../../src/text/mod.rs"]
mod text;
#[path = "../../src/upload.rs"]
mod upload;

const WIDTH: u32 = 768;
const HEIGHT: u32 = 424;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const FONTS: [&[u8]; 5] = [
    include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf"),
    include_bytes!("../../../../assets/fonts/NotoSansArabic.ttf"),
    include_bytes!("../../../../assets/fonts/NotoSansHebrew.ttf"),
    include_bytes!("../../../../assets/fonts/NotoEmoji-Regular.ttf"),
    include_bytes!("../../../../assets/fonts/FiraMono-Medium.ttf"),
];

struct Fixture {
    device: wgpu::Device,
    queue: wgpu::Queue,
    gpu: text::TextGpu,
    engine: TextEngine,
}

impl Fixture {
    /// Creates the production glyph pipeline and an isolated embedded-font engine.
    ///
    /// Returns a fixture with no OS window or system-font dependency.
    ///
    /// # Panics
    /// Panics if the requested native check cannot acquire a GPU adapter or device.
    fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .expect("text fidelity requires a working GPU adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        Self {
            gpu: text::TextGpu::new(&device, FORMAT),
            device,
            queue,
            engine: TextEngine::from_embedded_fonts(
                FONTS,
                "Noto Sans",
                "Noto Sans",
                "Fira Mono Medium",
            ),
        }
    }

    /// Renders `scene` and its `list` transforms over opaque `background` at `scale`.
    ///
    /// Returns packed, unpadded RGBA sRGB bytes for the scaled fixture viewport.
    ///
    /// # Panics
    /// Panics on GPU preparation, submission, or readback failure.
    fn render(
        &mut self,
        scene: &TextScene,
        list: &DisplayList,
        background: Color,
        scale: f32,
    ) -> Vec<u8> {
        let [width, height] = dimensions(scale);
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text-fidelity-output"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = output.create_view(&Default::default());
        let prepared = self.engine.prepare(scene, scale);
        self.gpu.begin_frame();
        let draw = self
            .gpu
            .prepare_ui(
                &self.device,
                &self.queue,
                &mut self.engine,
                &prepared,
                list,
                scale,
            )
            .unwrap();
        assert!(draw.ranges().iter().all(|range| !range.is_empty()));
        let viewport = self
            .gpu
            .target_offset(&self.queue, [0.0, 0.0, width as f32, height as f32]);
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let [red, green, blue, alpha] = background.to_linear_rgba().map(f64::from);
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text-fidelity-render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: red,
                            g: green,
                            b: blue,
                            a: alpha,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.gpu.draw(&mut pass, draw.all(), viewport);
        }
        read_pixels(&self.device, &self.queue, encoder, &output, extent)
    }
}

/// Returns the physical width and height for positive fixture DPI `scale`.
fn dimensions(scale: f32) -> [u32; 2] {
    [
        (WIDTH as f32 * scale) as u32,
        (HEIGHT as f32 * scale) as u32,
    ]
}

/// Returns a text scene and matching backdrop-aware commands for one theme.
///
/// `foreground` colors each text row; `background` supplies the known opaque
/// backdrop. `bake_transforms` moves the fractional translation into text bounds
/// so callers can compare shaping at the final position with GPU translation.
fn scene(foreground: Color, background: Color, bake_transforms: bool) -> (TextScene, DisplayList) {
    let rows = [
        (
            "12 px · Hamburgefontsiv 0123456789",
            12.0,
            16.0,
            "Noto Sans",
            0.0,
        ),
        (
            "16 px · Café, naïve, Ångström — office affine",
            16.0,
            44.0,
            "Noto Sans",
            0.0,
        ),
        (
            "Combining · e\u{301} a\u{308} n\u{303} A\u{30a} o\u{302}",
            18.0,
            76.0,
            "Noto Sans",
            0.0,
        ),
        (
            "العَرَبِيَّة — مرحبًا بالعالم ١٢٣",
            22.0,
            112.0,
            "Noto Sans Arabic",
            0.0,
        ),
        (
            "עברית — שלום עולם 123",
            22.0,
            156.0,
            "Noto Sans Hebrew",
            0.0,
        ),
        ("😀 🚀 ❤ ☕ ✨", 24.0, 200.0, "Noto Emoji", 0.0),
        (
            "let glyph = atlas.lookup(key); // 0123456789",
            14.0,
            240.0,
            "Fira Mono Medium",
            0.0,
        ),
        (
            "Fractional position · HAMBURG e\u{301} 0123456789",
            16.0,
            280.0,
            "Noto Sans",
            0.0,
        ),
        (
            "Fractional position · HAMBURG e\u{301} 0123456789",
            16.0,
            312.0,
            "Noto Sans",
            0.25,
        ),
        (
            "Fractional position · HAMBURG e\u{301} 0123456789",
            16.0,
            344.0,
            "Noto Sans",
            0.5,
        ),
        (
            "Fractional position · HAMBURG e\u{301} 0123456789",
            16.0,
            376.0,
            "Noto Sans",
            0.75,
        ),
    ];
    let mut scene = TextScene::new();
    let mut list = DisplayList::new();
    for (index, (content, size, top, family, offset)) in rows.into_iter().enumerate() {
        let baked = if bake_transforms { offset } else { 0.0 };
        let bounds = Rect::new(
            Point::new(24.0 + baked, top + baked),
            Size::new(720.0, 36.0),
        );
        scene.push(
            TextBlock::new(content, bounds)
                .size(size)
                .family(FontFamily::Named(family.to_owned()))
                .weight(if family == "Fira Mono Medium" {
                    500
                } else {
                    400
                })
                .color(foreground)
                .clip(Rect::new(
                    Point::default(),
                    Size::new(WIDTH as f32, HEIGHT as f32),
                )),
        );
        let translation = if bake_transforms { 0.0 } else { offset };
        list.push_text_with_backdrop(
            index,
            Affine2D::translation(translation, translation),
            ClipChain::default(),
            Some(background),
        );
    }
    (scene, list)
}

/// Submits `encoder`, copies the `output` texture at `extent`, and returns RGBA bytes.
///
/// `device` and `queue` must own both the encoder and output texture.
///
/// # Panics
/// Panics if the GPU copy, mapping, or completion callback fails.
fn read_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    output: &wgpu::Texture,
    extent: wgpu::Extent3d,
) -> Vec<u8> {
    let stride = (extent.width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("text-fidelity-readback"),
        size: u64::from(stride) * u64::from(extent.height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(extent.height),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = readback.slice(..).get_mapped_range().unwrap();
    mapped
        .chunks_exact(stride as usize)
        .flat_map(|row| row[..extent.width as usize * 4].iter().copied())
        .collect()
}

/// Rejects a capture if any expected glyph row contains too few contrasting pixels.
///
/// `pixels` is packed RGBA for `scale`; `name` identifies a failed capture.
///
/// # Panics
/// Panics on transparent output or a blank or missing script/translation row.
fn assert_visible(pixels: &[u8], scale: f32, name: &str) {
    let [width, _] = dimensions(scale);
    let background = &pixels[..4];
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 255)
    );
    for (start, end) in [
        (16, 42),
        (44, 74),
        (76, 110),
        (112, 154),
        (156, 198),
        (200, 238),
        (240, 278),
        (280, 310),
        (312, 342),
        (344, 374),
        (376, 412),
    ] {
        let top = (start as f32 * scale) as usize;
        let bottom = (end as f32 * scale) as usize;
        let row_bytes = width as usize * 4;
        let ink = pixels[top * row_bytes..bottom * row_bytes]
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| {
                pixel[..3]
                    .iter()
                    .zip(background)
                    .any(|(a, b)| a.abs_diff(*b) > 4)
            })
            .count();
        assert!(
            ink > (25.0 * scale * scale) as usize,
            "{name}: blank row at {start}"
        );
    }
}

/// Saves packed RGBA `pixels` at `scale` as a PNG at `path` without extra dependencies.
///
/// # Panics
/// Panics if dimensions are invalid or the output file cannot be written.
fn save_capture(path: &Path, pixels: Vec<u8>, scale: f32) {
    let [width, height] = dimensions(scale);
    let size = resvg::tiny_skia::IntSize::from_wh(width, height).unwrap();
    resvg::tiny_skia::Pixmap::from_vec(pixels, size)
        .unwrap()
        .save_png(path)
        .unwrap();
}

/// Captures every DPI/theme and checks warmed rendering against freshly positioned glyphs.
///
/// Enable with `ARGUI_NATIVE_TESTS=1` inside the private Linux display. PNGs go
/// to `ARGUI_TEXT_CAPTURE_DIR`, or workspace `target/text-fidelity` by default.
///
/// # Panics
/// Panics on missing GPU support, blank output, unstable rendering, or mismatched
/// translated/fresh glyph pixels.
#[test]
fn embedded_fonts_render_at_fractional_dpi_and_positions() {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }
    let directory = std::env::var_os("ARGUI_TEXT_CAPTURE_DIR").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/text-fidelity"),
        PathBuf::from,
    );
    std::fs::create_dir_all(&directory).unwrap();
    let mut fixture = Fixture::new();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        for (theme, foreground, background) in [
            (
                "light",
                Color::from_srgb8(30, 34, 42),
                Color::from_srgb8(246, 247, 250),
            ),
            (
                "dark",
                Color::from_srgb8(236, 240, 247),
                Color::from_srgb8(22, 29, 43),
            ),
            (
                "blue",
                Color::from_srgb8(255, 226, 128),
                Color::from_srgb8(37, 99, 235),
            ),
        ] {
            let name = format!("text-{theme}-{scale:.2}x.png");
            let (translated, list) = scene(foreground, background, false);
            let pixels = fixture.render(&translated, &list, background, scale);
            assert_visible(&pixels, scale, &name);
            save_capture(&directory.join(&name), pixels.clone(), scale);
            let warmed = fixture.render(&translated, &list, background, scale);
            assert!(pixels == warmed, "{name}: cached glyph rendering changed");
            let stats = fixture.gpu.stats();
            assert_eq!(
                stats.raster_requests_this_frame, 0,
                "{name}: warmed atlas rerasterized glyphs"
            );
            assert_eq!(
                stats.uploaded_bytes_this_frame, 0,
                "{name}: warmed atlas uploaded glyphs"
            );
            let (baked, baked_list) = scene(foreground, background, true);
            let positioned = fixture.render(&baked, &baked_list, background, scale);
            let differing_channels = pixels
                .iter()
                .zip(&positioned)
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(
                differing_channels, 0,
                "{name}: fractional transform blurred or displaced glyphs"
            );
        }
    }
}

/// Checks original COLRv0 glyph palettes, translucent edges, and the GPU sRGB path.
///
/// The generated 996-byte fixture has opaque blue and half-opaque red polygons.
/// Comparing raster samples against linear compositing catches missing sRGB
/// decoding, applying foreground tint to color glyphs, and double alpha.
/// A viewport clip keeps edge samples clear of independent clipping antialiasing.
#[test]
fn color_glyph_atlas_preserves_palette_and_straight_alpha() {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }
    let font = include_bytes!("assets/test-color.ttf").as_slice();
    let mut fixture = Fixture::new();
    fixture.engine = TextEngine::from_embedded_fonts(
        [font],
        "Argui Color Test",
        "Argui Color Test",
        "Argui Color Test",
    );
    let background = Color::from_srgb8(238, 224, 196);
    let scene = TextScene::new().with(
        TextBlock::new(
            "AB",
            Rect::new(Point::new(24.25, 16.0), Size::new(240.0, 96.0)),
        )
        .size(64.0)
        .family(FontFamily::Named("Argui Color Test".to_owned()))
        .color(Color::from_srgb8(0, 255, 0))
        .clip(Rect::new(
            Point::default(),
            Size::new(WIDTH as f32, HEIGHT as f32),
        )),
    );
    let mut list = DisplayList::new();
    list.push_text_with_backdrop(
        0,
        Affine2D::IDENTITY,
        ClipChain::default(),
        Some(background),
    );
    let prepared = fixture.engine.prepare(&scene, 1.0);
    assert_eq!(prepared.glyphs.len(), 2);
    let images: Vec<_> = prepared
        .glyphs
        .iter()
        .map(|glyph| fixture.engine.rasterize(glyph.key).unwrap())
        .collect();
    let pixels = fixture.render(&scene, &list, background, 1.0);
    let stats = fixture.gpu.stats();
    assert_eq!(stats.color_pages, 1);
    assert_eq!(stats.mask_pages, 0);
    assert_eq!(stats.allocated_bytes, 12 * 1024 * 1024);
    for ((glyph, image), palette) in prepared
        .glyphs
        .iter()
        .zip(&images)
        .zip([[51, 102, 204], [204, 51, 102]])
    {
        assert_eq!(image.content, argui_text::GlyphContent::Color);
        let center = ((image.height / 2 * image.width + image.width / 2) * 4) as usize;
        for (actual, expected) in image.data[center..center + 3].iter().zip(palette) {
            assert!(
                actual.abs_diff(expected) <= 4,
                "COLR color was not unpremultiplied: {actual} vs {expected}"
            );
        }
        assert!(
            image
                .data
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0 && pixel[3] < 200)
        );
        let backdrop = background.to_linear_rgba();
        for (offset, sample) in image
            .data
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, pixel)| pixel[3] > 0)
        {
            let source =
                Color::from_srgba8(sample[0], sample[1], sample[2], sample[3]).to_linear_rgba();
            let blended = std::array::from_fn::<_, 3, _>(|channel| {
                source[channel] * source[3] + backdrop[channel] * (1.0 - source[3])
            });
            let expected = Color::linear_rgba(blended[0], blended[1], blended[2], 1.0).to_srgba8();
            let x = glyph.x + image.left + (offset % image.width as usize) as i32;
            let y = glyph.y - image.top + (offset / image.width as usize) as i32;
            let output = (y as usize * WIDTH as usize + x as usize) * 4;
            for (actual, expected) in pixels[output..output + 4].iter().zip(expected) {
                assert!(
                    actual.abs_diff(expected) <= 2,
                    "color glyph compositing changed at ({x}, {y}): {actual} vs {expected}"
                );
            }
        }
    }
}

#[path = "mod.rs"]
mod clipping;
