use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockResult};

/// Trait for structs that hold some data
pub trait DataHolder {
    /// The type of the data
    type Data;

    /// Create a holder for some data
    fn new(data: Self::Data) -> Self;

    /// Get a reference to the inner data
    fn inner(&self) -> &Arc<RwLock<Self::Data>>;

    /// Lock for reading
    fn read(&self) -> TryLockResult<RwLockReadGuard<'_, Self::Data>> {
        self.inner().try_read()
    }

    /// Lock for writing
    fn write(&self) -> TryLockResult<RwLockWriteGuard<'_, Self::Data>> {
        self.inner().try_write()
    }
}
