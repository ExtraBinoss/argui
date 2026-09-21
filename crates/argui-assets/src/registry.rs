use std::collections::HashMap;

use argui_paint::{ImageAsset, ImageId, VectorAsset, VectorId};

use crate::{AssetKey, AssetRegistryError, AssetRevision};

/// Runtime representation stored under a stable source key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetKind {
    Image,
    Vector,
}

/// Stable renderer handle associated with a source asset.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AssetHandle {
    Image(ImageId),
    Vector(VectorId),
}

/// Current decoded content and metadata for one stable source key.
#[derive(Clone, Debug, PartialEq)]
pub enum AssetRecord {
    Image {
        key: AssetKey,
        revision: AssetRevision,
        content_hash: u64,
        asset: ImageAsset,
    },
    Vector {
        key: AssetKey,
        revision: AssetRevision,
        content_hash: u64,
        asset: VectorAsset,
    },
}

impl AssetRecord {
    /// Returns this record's canonical source key.
    #[must_use]
    pub const fn key(&self) -> &AssetKey {
        match self {
            Self::Image { key, .. } | Self::Vector { key, .. } => key,
        }
    }

    /// Returns the current content revision.
    #[must_use]
    pub const fn revision(&self) -> AssetRevision {
        match self {
            Self::Image { revision, .. } | Self::Vector { revision, .. } => *revision,
        }
    }

    /// Returns the stable renderer handle retained across revisions.
    #[must_use]
    pub const fn handle(&self) -> AssetHandle {
        match self {
            Self::Image { asset, .. } => AssetHandle::Image(asset.id),
            Self::Vector { asset, .. } => AssetHandle::Vector(asset.id),
        }
    }

    /// Returns the asset's decoded kind.
    #[must_use]
    pub const fn kind(&self) -> AssetKind {
        match self {
            Self::Image { .. } => AssetKind::Image,
            Self::Vector { .. } => AssetKind::Vector,
        }
    }

    fn content_hash(&self) -> u64 {
        match self {
            Self::Image { content_hash, .. } | Self::Vector { content_hash, .. } => *content_hash,
        }
    }
}

/// Result of a source asset upsert.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetChange {
    pub key: AssetKey,
    pub kind: AssetKind,
    pub revision: AssetRevision,
    pub handle: AssetHandle,
    pub changed: bool,
}

/// Transactional source-to-runtime asset registry.
#[derive(Clone, Debug, Default)]
pub struct AssetRegistry {
    records: HashMap<AssetKey, AssetRecord>,
}

impl AssetRegistry {
    /// Creates an empty source asset registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Decodes and inserts or updates an image while retaining its logical handle.
    ///
    /// * `key` — canonical source identity.
    /// * `encoded` — complete encoded PNG/JPEG bytes.
    ///
    /// # Errors
    ///
    /// Returns an error without modifying the prior record when decoding fails,
    /// the key is already a vector, or revision space is exhausted.
    pub fn upsert_image(
        &mut self,
        key: AssetKey,
        encoded: &[u8],
    ) -> Result<AssetChange, AssetRegistryError> {
        let hash = content_hash(encoded);
        if let Some(record) = self.records.get(&key) {
            if record.kind() != AssetKind::Image {
                return Err(kind_mismatch(&key, record.kind(), AssetKind::Image));
            }
            if record.content_hash() == hash {
                return Ok(change(record, false));
            }
        }
        let (id, revision) = match self.records.get(&key) {
            Some(AssetRecord::Image {
                asset, revision, ..
            }) => (asset.id, revision.next(&key)?),
            Some(_) => return Err(kind_mismatch(&key, AssetKind::Vector, AssetKind::Image)),
            None => (ImageId::fresh(), AssetRevision::INITIAL),
        };
        let asset =
            argui_image::decode(id, encoded).map_err(|source| AssetRegistryError::Image {
                key: key.clone(),
                source,
            })?;
        let record = AssetRecord::Image {
            key: key.clone(),
            revision,
            content_hash: hash,
            asset,
        };
        let result = change(&record, true);
        self.records.insert(key, record);
        Ok(result)
    }

    /// Parses and inserts or updates an SVG while retaining its logical handle.
    ///
    /// * `key` — canonical source identity.
    /// * `svg` — complete SVG document bytes.
    ///
    /// # Errors
    ///
    /// Returns an error without modifying the prior record when parsing fails,
    /// the key is already an image, or revision space is exhausted.
    pub fn upsert_vector(
        &mut self,
        key: AssetKey,
        svg: &[u8],
    ) -> Result<AssetChange, AssetRegistryError> {
        let hash = content_hash(svg);
        if let Some(record) = self.records.get(&key) {
            if record.kind() != AssetKind::Vector {
                return Err(kind_mismatch(&key, record.kind(), AssetKind::Vector));
            }
            if record.content_hash() == hash {
                return Ok(change(record, false));
            }
        }
        let (id, revision) = match self.records.get(&key) {
            Some(AssetRecord::Vector {
                asset, revision, ..
            }) => (asset.id, revision.next(&key)?),
            Some(_) => return Err(kind_mismatch(&key, AssetKind::Image, AssetKind::Vector)),
            None => (VectorId::fresh(), AssetRevision::INITIAL),
        };
        let asset =
            argui_vector::parse_svg(id, svg).map_err(|source| AssetRegistryError::Vector {
                key: key.clone(),
                source,
            })?;
        let record = AssetRecord::Vector {
            key: key.clone(),
            revision,
            content_hash: hash,
            asset,
        };
        let result = change(&record, true);
        self.records.insert(key, record);
        Ok(result)
    }

    /// Returns the current decoded record for `key`.
    #[must_use]
    pub fn get(&self, key: &AssetKey) -> Option<&AssetRecord> {
        self.records.get(key)
    }

    /// Iterates over records in canonical key order.
    pub fn records(&self) -> impl Iterator<Item = &AssetRecord> {
        let mut records = self.records.values().collect::<Vec<_>>();
        records.sort_unstable_by(|left, right| left.key().cmp(right.key()));
        records.into_iter()
    }

    /// Removes a source record and returns its final decoded generation.
    pub fn remove(&mut self, key: &AssetKey) -> Option<AssetRecord> {
        self.records.remove(key)
    }

    /// Returns the number of registered source assets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether no source assets are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

fn change(record: &AssetRecord, changed: bool) -> AssetChange {
    AssetChange {
        key: record.key().clone(),
        kind: record.kind(),
        revision: record.revision(),
        handle: record.handle(),
        changed,
    }
}

fn kind_mismatch(key: &AssetKey, existing: AssetKind, requested: AssetKind) -> AssetRegistryError {
    AssetRegistryError::KindMismatch {
        key: key.clone(),
        existing,
        requested,
    }
}

fn content_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
