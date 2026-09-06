use std::{collections::HashSet, time::Duration};

use argui_core::Rect;

use crate::{NativeWebView, WebViewBackend, WebViewError, WebViewId, WebViewState};

#[derive(Clone, Copy, Debug)]
pub struct PoolConfig {
    pub max_resident: usize,
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_resident: 2,
            idle_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PoolStats {
    pub resident: usize,
    pub created: u64,
    pub reused: u64,
    pub evicted: u64,
}

#[derive(Clone, Debug)]
pub struct WebViewMount {
    pub state: WebViewState,
    pub host: u64,
    pub bounds: Rect,
    /// Covered by an Argui overlay: retain the instance but remove native input/paint.
    pub occluded: bool,
}

struct Resident<V> {
    state: WebViewState,
    host: u64,
    view: V,
    bounds: Rect,
    visible: bool,
    revision: u64,
    release_epoch: u64,
    focus: u64,
    last_used: Duration,
    idle_since: Option<Duration>,
}

pub struct WebViewPool<B: WebViewBackend> {
    backend: B,
    config: PoolConfig,
    residents: Vec<Resident<B::View>>,
    stats: PoolStats,
}

impl<B: WebViewBackend> WebViewPool<B> {
    /// Access host-specific integration without transferring ownership of resident views.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn new(backend: B, config: PoolConfig) -> Result<Self, WebViewError> {
        if config.max_resident == 0 {
            return Err(WebViewError::CapacityExceeded);
        }
        Ok(Self {
            backend,
            config,
            residents: Vec::new(),
            stats: PoolStats::default(),
        })
    }

    #[must_use]
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            resident: self.residents.len(),
            ..self.stats
        }
    }

    #[must_use]
    pub fn contains_pointer(&self, point: argui_core::Point) -> bool {
        self.residents
            .iter()
            .any(|resident| resident.visible && resident.bounds.contains(point))
    }

    /// One call after layout/commands change, or when `next_expiry` elapses.
    /// Unchanged slots do not call the native backend.
    pub fn reconcile(
        &mut self,
        mounts: &[WebViewMount],
        now: Duration,
    ) -> Result<(), WebViewError> {
        let mut seen = HashSet::new();
        for mount in mounts {
            if !seen.insert(mount.state.id()) {
                return Err(WebViewError::DuplicateMount);
            }
            if !valid_bounds(mount.bounds) {
                return Err(WebViewError::InvalidBounds);
            }
        }
        let active: HashSet<_> = mounts
            .iter()
            .filter(|mount| active_mount(mount))
            .map(|mount| mount.state.id())
            .collect();
        if active.len() > self.config.max_resident {
            return Err(WebViewError::CapacityExceeded);
        }
        let mut index = 0;
        while index < self.residents.len() {
            let resident = &mut self.residents[index];
            let id = resident.state.id();
            let host_changed = mounts
                .iter()
                .any(|mount| mount.state.id() == id && mount.host != resident.host);
            if resident.state.released()
                || resident.release_epoch != resident.state.release_epoch()
                || host_changed
            {
                self.evict(index);
                continue;
            }
            if !active.contains(&id) {
                if resident.visible {
                    resident.view.set_visible(false)?;
                    resident.visible = false;
                }
                let since = *resident.idle_since.get_or_insert(now);
                if now.saturating_sub(since) >= self.config.idle_timeout {
                    self.evict(index);
                    continue;
                }
            }
            index += 1;
        }
        for mount in mounts.iter().filter(|mount| active_mount(mount)) {
            let index = match self.index(mount.state.id()) {
                Some(index) => index,
                None => {
                    if self.residents.len() == self.config.max_resident {
                        let oldest = self
                            .residents
                            .iter()
                            .enumerate()
                            .filter(|(_, resident)| !active.contains(&resident.state.id()))
                            .min_by_key(|(_, resident)| resident.last_used)
                            .map(|(index, _)| index)
                            .ok_or(WebViewError::CapacityExceeded)?;
                        self.evict(oldest);
                    }
                    let result: Result<B::View, WebViewError> = (|| {
                        let mut view = self.backend.create(
                            mount.host,
                            &mount.state,
                            mount.state.new_sink(),
                        )?;
                        self.stats.created += 1;
                        view.set_bounds(mount.bounds)?;
                        view.load(&mount.state.source())?;
                        Ok(view)
                    })();
                    let view = match result {
                        Ok(view) => view,
                        Err(error) => {
                            mount.state.fail(error.clone());
                            return Err(error);
                        }
                    };
                    self.residents.push(Resident {
                        state: mount.state.clone(),
                        host: mount.host,
                        view,
                        bounds: mount.bounds,
                        visible: false,
                        revision: mount.state.revision(),
                        release_epoch: mount.state.release_epoch(),
                        focus: 0,
                        last_used: now,
                        idle_since: None,
                    });
                    self.residents.len() - 1
                }
            };
            let resident = &mut self.residents[index];
            if resident.idle_since.take().is_some() {
                self.stats.reused += 1;
            }
            if resident.revision != mount.state.revision() {
                resident.view.load(&mount.state.source())?;
                resident.revision = mount.state.revision();
                self.stats.reused += 1;
            }
            if resident.bounds != mount.bounds {
                resident.view.set_bounds(mount.bounds)?;
                resident.bounds = mount.bounds;
            }
            if resident.visible == mount.occluded {
                resident.view.set_visible(!mount.occluded)?;
                resident.visible = !mount.occluded;
            }
            if resident.visible && resident.focus != mount.state.focus_revision() {
                resident.view.focus()?;
                resident.focus = mount.state.focus_revision();
            }
            resident.last_used = now;
        }
        Ok(())
    }

    /// Monotonic deadline used to wake an otherwise idle event loop.
    #[must_use]
    pub fn next_expiry(&self) -> Option<Duration> {
        self.residents
            .iter()
            .filter_map(|resident| resident.idle_since)
            .filter_map(|since| since.checked_add(self.config.idle_timeout))
            .min()
    }

    pub fn close_host(&mut self, host: u64) {
        while let Some(index) = self
            .residents
            .iter()
            .position(|resident| resident.host == host)
        {
            self.evict(index);
        }
    }

    fn index(&self, id: WebViewId) -> Option<usize> {
        self.residents
            .iter()
            .position(|resident| resident.state.id() == id)
    }

    fn evict(&mut self, index: usize) {
        let resident = self.residents.remove(index);
        resident.state.evict();
        drop(resident);
        self.stats.evicted += 1;
    }
}

impl<B: WebViewBackend> Drop for WebViewPool<B> {
    fn drop(&mut self) {
        for resident in &self.residents {
            resident.state.evict();
        }
    }
}

fn active_mount(mount: &WebViewMount) -> bool {
    !mount.state.released() && mount.bounds.size.width > 0.0 && mount.bounds.size.height > 0.0
}

fn valid_bounds(bounds: Rect) -> bool {
    [
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
    ]
    .iter()
    .all(|value| value.is_finite())
        && bounds.size.width >= 0.0
        && bounds.size.height >= 0.0
}
