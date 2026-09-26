mod app {
    mod frame_route;
    mod inertia;
    #[cfg(feature = "inspect")]
    mod inspect;
    mod observations;
    mod semantic_sync;
    mod scroll {
        mod request;
    }
    mod touch_scroll;
    mod zoom;
}
