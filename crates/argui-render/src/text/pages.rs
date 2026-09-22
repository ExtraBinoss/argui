/// One allocated glyph rectangle, including a one-pixel transparent border.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Allocation {
    pub page: usize,
    pub origin: [u32; 2],
    pub evicted: bool,
}

#[derive(Clone, Default)]
struct Page {
    cursor: [u32; 2],
    row_height: u32,
    last_used: u64,
    pinned: bool,
}

/// Bounded shelf pages with frame-pinned, least-recently-used eviction.
pub(super) struct AtlasPages {
    pages: Vec<Page>,
    size: u32,
    frame: u64,
}

impl AtlasPages {
    /// Creates `count` empty square pages whose sides contain `size` texels.
    pub fn new(size: u32, count: usize) -> Self {
        Self {
            pages: vec![Page::default(); count],
            size,
            frame: 0,
        }
    }

    /// Starts a new frame, allowing old pages to be evicted until pinned again.
    pub fn begin_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
        for page in &mut self.pages {
            page.pinned = false;
        }
    }

    /// Protects `index` for this frame and records its recent use; ignores absent pages.
    pub fn pin(&mut self, index: usize) {
        if let Some(page) = self.pages.get_mut(index) {
            page.pinned = true;
            page.last_used = self.frame;
        }
    }

    /// Allocates `width` by `height` glyph pixels with a transparent border.
    ///
    /// Returns `None` if the glyph is oversized or every reusable page is pinned.
    /// An evicted allocation requires callers to remove that page's old entries.
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<Allocation> {
        let width = width.checked_add(2)?;
        let height = height.checked_add(2)?;
        if width > self.size || height > self.size {
            return None;
        }
        for index in 0..self.pages.len() {
            if let Some(origin) = self.reserve(index, width, height) {
                return Some(Allocation {
                    page: index,
                    origin,
                    evicted: false,
                });
            }
        }
        let index = self
            .pages
            .iter()
            .enumerate()
            .filter(|(_, page)| !page.pinned)
            .min_by_key(|(_, page)| page.last_used)?
            .0;
        self.pages[index] = Page::default();
        let origin = self.reserve(index, width, height)?;
        Some(Allocation {
            page: index,
            origin,
            evicted: true,
        })
    }

    /// Returns the number of pages populated since creation or their last eviction.
    pub fn used(&self) -> usize {
        self.pages
            .iter()
            .filter(|page| page.row_height != 0)
            .count()
    }

    /// Reserves padded `width` by `height` texels in `index`, returning their origin.
    /// A failed reservation leaves the shelf unchanged.
    fn reserve(&mut self, index: usize, width: u32, height: u32) -> Option<[u32; 2]> {
        let page = &mut self.pages[index];
        let new_row = page.cursor[0] + width > self.size;
        let x = if new_row { 0 } else { page.cursor[0] };
        let y = if new_row {
            page.cursor[1] + page.row_height
        } else {
            page.cursor[1]
        };
        if y + height > self.size {
            return None;
        }
        page.cursor = [x + width, y];
        page.row_height = if new_row {
            height
        } else {
            page.row_height.max(height)
        };
        self.pin(index);
        Some([x, y])
    }
}
