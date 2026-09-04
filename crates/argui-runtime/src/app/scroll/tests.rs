use std::time::Duration;

use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_layout::{LayoutNode, LayoutOutput};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{
    Element, InertialScroll, ScrollAlignment, ScrollConfig, ScrollPhysics, ScrollRegion,
    ScrollRequest, UiTree,
};
use web_time::Instant;
use winit::event::TouchPhase;

use super::{
    ScrollInertia, ScrollSample, merge_delta,
    request::{align_axis, scroll_tracks},
};

#[test]
fn pointer_scroll_bursts_coalesce_without_losing_distance_or_units() {
    let mut pixels = ScrollDelta::Pixels(Point::new(2.0, 4.0));
    assert!(merge_delta(
        &mut pixels,
        ScrollDelta::Pixels(Point::new(-1.0, 8.0))
    ));
    assert_eq!(pixels, ScrollDelta::Pixels(Point::new(1.0, 12.0)));
    assert!(!merge_delta(
        &mut pixels,
        ScrollDelta::Lines(Point::new(0.0, 1.0))
    ));
    let mut lines = ScrollDelta::Lines(Point::new(1.0, 2.0));
    assert!(merge_delta(
        &mut lines,
        ScrollDelta::Lines(Point::new(3.0, 4.0))
    ));
    assert_eq!(lines, ScrollDelta::Lines(Point::new(4.0, 6.0)));
}

#[test]
fn pixel_scrolls_gain_a_short_tail_while_wheel_lines_stay_direct() {
    let start = Instant::now();
    let mut inertia = ScrollInertia::default();
    inertia.observe(sample(
        ScrollDelta::Pixels(Point::new(0.0, -8.0)),
        TouchPhase::Moved,
        start,
    ));
    let tail = inertia
        .advance(start + Duration::from_millis(24))
        .expect("pixel input starts inertia after the quiet period")
        .0;
    assert!(tail.y < 0.0);
    assert!(inertia.needs_frame());

    inertia.observe(sample(
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        TouchPhase::Moved,
        start,
    ));
    assert!(!inertia.needs_frame());
}

#[test]
fn inertia_handles_gesture_phases_quiet_period_and_settling() {
    let start = Instant::now();
    let mut inertia = ScrollInertia::default();
    assert_eq!(inertia.advance(start), None);

    inertia.observe(sample(
        ScrollDelta::Pixels(Point::new(0.0, -12.0)),
        TouchPhase::Started,
        start,
    ));
    assert_eq!(
        inertia.advance(start + Duration::from_millis(8)),
        None,
        "an active gesture has no synthetic motion before the quiet period"
    );
    inertia.observe(sample(
        ScrollDelta::Pixels(Point::new(0.0, -4.0)),
        TouchPhase::Moved,
        start + Duration::from_millis(10),
    ));
    inertia.observe(sample(
        ScrollDelta::Pixels(Point::default()),
        TouchPhase::Ended,
        start + Duration::from_millis(12),
    ));
    assert!(inertia.advance(start + Duration::from_millis(16)).is_some());

    let mut stopped = ScrollInertia::default();
    stopped.observe(sample(
        ScrollDelta::Pixels(Point::default()),
        TouchPhase::Ended,
        start,
    ));
    assert_eq!(stopped.advance(start + Duration::from_millis(20)), None);
    assert!(!stopped.needs_frame());

    inertia.observe(sample(
        ScrollDelta::Pixels(Point::new(0.0, 3.0)),
        TouchPhase::Cancelled,
        start,
    ));
    assert!(!inertia.needs_frame());
}

#[test]
fn physics_modes_and_touch_dispatch_are_explicit() {
    let start = Instant::now();
    for physics in [ScrollPhysics::Direct, ScrollPhysics::Native] {
        let mut inertia = ScrollInertia::default();
        let mut input = sample(
            ScrollDelta::Pixels(Point::new(0.0, -8.0)),
            TouchPhase::Moved,
            start,
        );
        input.physics = physics;
        inertia.observe(input);
        assert!(!inertia.needs_frame());
    }

    let mut touch = ScrollInertia::default();
    let mut started = sample(
        ScrollDelta::Pixels(Point::new(0.0, -8.0)),
        TouchPhase::Started,
        start,
    );
    started.dispatch_wheel = false;
    touch.observe(started);
    let mut ended = sample(
        ScrollDelta::Pixels(Point::default()),
        TouchPhase::Ended,
        start,
    );
    ended.dispatch_wheel = false;
    touch.observe(ended);
    assert!(!touch.advance(start + Duration::from_millis(8)).unwrap().2);

    let mut invalid = ScrollInertia::default();
    let mut input = sample(
        ScrollDelta::Pixels(Point::new(0.0, -8.0)),
        TouchPhase::Moved,
        start,
    );
    input.physics = ScrollPhysics::Inertial(InertialScroll {
        velocity_limit: f32::NAN,
        stop_velocity: f32::NAN,
        decay: f32::NAN,
        sample_weight: f32::NAN,
        ..InertialScroll::default()
    });
    invalid.observe(input);
    let (delta, _, dispatch_wheel) = invalid
        .advance(start + Duration::from_millis(24))
        .expect("invalid tuning values use the valid inertial defaults");
    assert!(delta.y.is_finite());
    assert!(dispatch_wheel);
}

#[test]
fn inertial_distance_is_stable_across_monitor_refresh_rates() {
    let distance = |hz: u32| {
        let start = Instant::now();
        let mut inertia = ScrollInertia::default();
        inertia.observe(sample(
            ScrollDelta::Pixels(Point::new(0.0, 12.0)),
            TouchPhase::Started,
            start,
        ));
        inertia.observe(sample(
            ScrollDelta::Pixels(Point::default()),
            TouchPhase::Ended,
            start,
        ));
        let frame = Duration::from_secs_f64(1.0 / f64::from(hz));
        let mut now = start;
        let mut total = 0.0;
        while inertia.needs_frame() {
            now += frame;
            total += inertia.advance(now).map_or(0.0, |(delta, _, _)| delta.y);
        }
        total
    };

    let at_60 = distance(60);
    for hz in [120, 144] {
        let candidate = distance(hz);
        assert!(
            (candidate - at_60).abs() < 0.75,
            "{hz} Hz: {candidate} vs {at_60}"
        );
    }
}

fn sample(delta: ScrollDelta, phase: TouchPhase, now: Instant) -> ScrollSample {
    ScrollSample {
        delta,
        phase,
        now,
        target: None,
        physics: ScrollPhysics::Hybrid,
        point: Point::default(),
        dispatch_wheel: true,
    }
}

#[test]
fn reveal_requests_use_nearest_alignment_without_moving_visible_targets() {
    let ui = UiTree::new(
        Element::column([
            Element::container([]).keyed("visible"),
            Element::container([]).keyed("below"),
        ])
        .keyed("container"),
    );
    let container = ui.node_id_at(0).unwrap();
    let visible = ui.node_id_at(1).unwrap();
    let below = ui.node_id_at(2).unwrap();
    let viewport = Rect::new(Point::default(), Size::new(100.0, 100.0));
    let layout = LayoutOutput {
        nodes: vec![
            layout_node(container, viewport),
            layout_node(
                visible,
                Rect::new(Point::new(0.0, 20.0), Size::new(100.0, 20.0)),
            ),
            layout_node(
                below,
                Rect::new(Point::new(0.0, 180.0), Size::new(100.0, 20.0)),
            ),
        ],
        scroll_regions: vec![scroll_region(container, viewport, 300.0)],
        viewport,
        ..LayoutOutput::default()
    };

    let visible_track = scroll_tracks(&ui, &layout, &ScrollRequest::reveal("visible"));
    assert_eq!(visible_track[0].to, Point::default());
    let below_track = scroll_tracks(&ui, &layout, &ScrollRequest::reveal("below"));
    assert_eq!(below_track[0].to, Point::new(0.0, 100.0));
    let offset = scroll_tracks(
        &ui,
        &layout,
        &ScrollRequest::offset("container", Point::new(0.0, 999.0)),
    );
    assert_eq!(offset[0].to, Point::new(0.0, 300.0));
    let rect = scroll_tracks(
        &ui,
        &layout,
        &ScrollRequest::rect(
            "container",
            Rect::new(Point::new(0.0, 250.0), Size::new(100.0, 20.0)),
        ),
    );
    assert_eq!(rect[0].to, Point::new(0.0, 170.0));
    assert!(scroll_tracks(&ui, &layout, &ScrollRequest::reveal("missing")).is_empty());
}

#[test]
fn axis_alignment_matches_start_center_end_and_nearest() {
    assert_eq!(
        align_axis(10.0, 0.0, 100.0, 40.0, 60.0, ScrollAlignment::Start),
        50.0
    );
    assert_eq!(
        align_axis(10.0, 0.0, 100.0, 40.0, 60.0, ScrollAlignment::Center),
        10.0
    );
    assert_eq!(
        align_axis(10.0, 0.0, 100.0, 40.0, 60.0, ScrollAlignment::End),
        -30.0
    );
    assert_eq!(
        align_axis(10.0, 0.0, 100.0, 40.0, 60.0, ScrollAlignment::Nearest),
        10.0
    );
}

fn layout_node(node: argui_ui::NodeId, bounds: Rect) -> LayoutNode {
    LayoutNode {
        index: 0,
        node,
        bounds,
        layout_bounds: bounds,
        clip: Some(bounds),
        text_index: None,
    }
}

fn scroll_region(node: argui_ui::NodeId, bounds: Rect, max_y: f32) -> ScrollRegion {
    ScrollRegion {
        node,
        bounds,
        clip: bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        max_offset: Point::new(0.0, max_y),
        config: ScrollConfig::default(),
        scrollbar: None,
        interaction_order: 0,
    }
}
