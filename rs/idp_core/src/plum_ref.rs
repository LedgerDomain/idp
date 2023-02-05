use crate::Datacache;
use idp_proto::PlumHeadSeal;
use std::{
    any::Any,
    sync::{Arc, RwLock, Weak},
};

/// A value specified by its PlumHeadSeal, which is loaded, deserialized, and cached into memory
/// when requested.  The Datacache stores Arc<T> instances, so there's no duplication of cached
/// values.  The cached value can be cleared, which would of course cause it to be re-loaded and
/// re-cached if the value were to be requested again.
pub struct PlumRef<T> {
    /// The address of the PlumRef itself.
    pub head_seal: PlumHeadSeal,
    /// Handle to the Datahost this PlumRef is scoped to.
    datacache_la: Arc<RwLock<Datacache>>,
    /// Weak pointer to the cached value.  The choice of Weak for this variable serves two purposes:
    /// - Allow the PlumRef to begin in a "non-linked" state (regardless of if it's cached in Datacache)
    /// - Allow the Datacache to reclaim memory if needed, by dropping the cached Arc<T>, after
    ///   which attempting to upgrade the Weak pointer will fail and cause the value to be re-cached.
    /// NOTE: This uses std::cell::UnsafeCell, and assumes single-threaded operation.  To have proper
    /// multi-threaded operation, probably some sort of Mutex/RwLock would be needed.  For now, UnsafeCell
    /// was chosen to avoid having too many layers in the type of this variable (i.e.
    /// `Arc<RwLock<Weak<T>>>`).
    value_wu: std::cell::UnsafeCell<Weak<T>>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Any + std::fmt::Debug + serde::de::DeserializeOwned> PlumRef<T> {
    pub fn new(head_seal: PlumHeadSeal, datacache_la: Arc<RwLock<Datacache>>) -> Self {
        Self {
            head_seal,
            datacache_la,
            value_wu: Default::default(),
            phantom: Default::default(),
        }
    }
    pub fn value_is_cached(&self) -> bool {
        unsafe { &*self.value_wu.get() }.strong_count() > 0
    }
    pub fn clear_cached_value(&self) {
        self.datacache_la
            .read()
            .unwrap()
            .clear_cached_value(&self.head_seal);
    }
    /// The only reason this returns `Result<Arc<T>, _>` instead of `Arc<T>` directly is because
    /// it calls Datacache::load_typed_content internally, which can fail because of connectivity
    /// or serialization errors.
    pub fn value(&self) -> Result<Arc<T>, failure::Error> {
        let value_a = if let Some(value_a) = unsafe { &*self.value_wu.get() }.upgrade() {
            // If the Weak pointer could be upgraded (i.e. the cached value was already linked),
            // then simply return that.
            value_a
        } else {
            // Otherwise the Weak pointer is not linked, so load the content from Datacache and link it.
            let value_a = self
                .datacache_la
                .read()
                .unwrap()
                .get_or_load_value::<T>(&self.head_seal)?;
            unsafe { *self.value_wu.get() = Arc::downgrade(&value_a) };
            value_a
        };
        Ok(value_a)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for PlumRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("PlumRef")
            .field("head_seal", &self.head_seal)
            .field("value_wu", unsafe { &*self.value_wu.get() })
            .finish()
    }
}
