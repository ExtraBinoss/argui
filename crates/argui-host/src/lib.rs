//! Atomic mutations from presentation adapters into Argui's retained native tree.

mod protocol;
mod transaction;

pub use protocol::{CallbackDelivery, CallbackId, HostId, Operation};
pub use transaction::{CommitResult, Host, HostError};
