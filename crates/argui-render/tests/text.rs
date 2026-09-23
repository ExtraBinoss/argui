#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[path = "text/pages.rs"]
mod page_tests;
#[path = "../src/text/pages.rs"]
mod pages;
#[path = "text/pixels.rs"]
mod pixel_tests;
#[path = "../src/text/pixels.rs"]
mod pixels;
