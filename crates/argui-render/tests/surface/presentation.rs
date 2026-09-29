use super::*;
use wgpu::PresentMode;

#[test]
fn automatic_vsync_prefers_supported_tear_free_mailbox() {
    assert_eq!(
        mailbox_present_mode(
            PresentMode::AutoVsync,
            &[PresentMode::Fifo, PresentMode::Mailbox]
        ),
        PresentMode::Mailbox,
    );
}

#[test]
fn unavailable_mailbox_keeps_automatic_vsync() {
    for supported in [&[][..], &[PresentMode::Fifo][..]] {
        assert_eq!(
            mailbox_present_mode(PresentMode::AutoVsync, supported),
            PresentMode::AutoVsync
        );
    }
}

#[test]
fn explicit_present_modes_remain_authoritative() {
    for requested in [
        PresentMode::Fifo,
        PresentMode::FifoRelaxed,
        PresentMode::Mailbox,
        PresentMode::Immediate,
        PresentMode::AutoNoVsync,
    ] {
        assert_eq!(
            mailbox_present_mode(requested, &[PresentMode::Fifo, PresentMode::Mailbox]),
            requested
        );
    }
}
