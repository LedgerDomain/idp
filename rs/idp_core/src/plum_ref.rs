use crate::Datacache;
use anyhow::Result;
use idp_proto::PlumHeadSeal;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

lazy_static::lazy_static! {
    pub static ref DATACACHE_OLA: Arc<RwLock<Option<Datacache>>> = Arc::new(RwLock::new(None));
}

pub fn initialize_datacache(datacache: Datacache) {
    *DATACACHE_OLA.write() = Some(datacache);
}

pub fn datacache() -> MappedRwLockReadGuard<'static, Datacache> {
    let datacache_og = DATACACHE_OLA.read();
    if datacache_og.is_none() {
        panic!("programmer error: DATACACHE_OLA has not been initialized");
    }
    RwLockReadGuard::map(datacache_og, |datacache_o| datacache_o.as_ref().unwrap())
}

/// A value specified by its PlumHeadSeal, which is loaded, deserialized, and cached into memory
/// when requested.  The Datacache stores Arc<T> instances, so there's no duplication of cached
/// values.  The cached value can be cleared, which would of course cause it to be re-loaded and
/// re-cached if the value were to be requested again.  NOTE: This uses a global Datacache, and
/// therefore is potentially a security vulnerability, as an attacker that can interact with the
/// program could use a timing attack to see if any particular Plums have been cached.  There are
/// mitigations to this, including:
/// -   Include a nonce in the PlumBody so that the PlumBodySeal (and therefore PlumHeadSeal) are
///     not a function of the body content alone.
/// -   Store `datacache_la: Arc<RwLock<Datacache>>` in the PlumRef itself.  This would require
///     a stateful deserialization routine.  See https://github.com/serde-rs/serde/issues/2212
///     and https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html
// TODO: Consider making the Weak<T> into a Weak<Content>, where Content stores UntypedValue and
// PlumHeadSeal, then get rid of PlumRef::head_seal, and change the guts of UnsafeCell to an
// enum whose variants are PlumHeadSeal and Weak<Content>.  This way, the overall data structure
// can stay relatively small, instead of being at least as large as PlumHeadSeal (256 bits).
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlumRef<T> {
    /// The address of the PlumRef itself.
    pub head_seal: PlumHeadSeal,
    /// Weak pointer to the cached value.  The choice of Weak for this variable serves two purposes:
    /// - Allow the PlumRef to begin in a "non-linked" state (regardless of if it's cached in Datacache)
    /// - Allow the Datacache to reclaim memory if needed, by dropping the cached Arc<T>, after
    ///   which attempting to upgrade the Weak pointer will fail and cause the value to be re-cached.
    /// NOTE: This uses std::cell::UnsafeCell, and assumes single-threaded operation.  To have proper
    /// multi-threaded operation, probably some sort of Mutex/RwLock would be needed.  For now, UnsafeCell
    /// was chosen to avoid having too many layers in the type of this variable (i.e.
    /// `Arc<RwLock<Weak<T>>>`).
    #[serde(skip)]
    value_wu: std::cell::UnsafeCell<Weak<T>>,
    #[serde(skip)]
    phantom: std::marker::PhantomData<T>,
}

impl<T: Any + std::fmt::Debug + serde::de::DeserializeOwned + Send + Sync> PlumRef<T> {
    pub fn new(head_seal: PlumHeadSeal) -> Self {
        Self {
            head_seal,
            value_wu: Default::default(),
            phantom: Default::default(),
        }
    }
    pub fn value_is_cached(&self) -> bool {
        unsafe { &*self.value_wu.get() }.strong_count() > 0
    }
    pub fn clear_cached_value(&self) {
        datacache().clear_cached_value(&self.head_seal);
    }
    /// The only reason this returns `Result<Arc<T>, _>` instead of `Arc<T>` directly is because
    /// it calls Datacache::load_typed_content internally, which can fail because of connectivity
    /// or serialization errors.
    pub fn value(&self) -> Result<Arc<T>> {
        let value_a = if let Some(value_a) = unsafe { &*self.value_wu.get() }.upgrade() {
            // If the Weak pointer could be upgraded (i.e. the cached value was already linked),
            // then simply return that.
            value_a
        } else {
            // Otherwise the Weak pointer is not linked, so load the content from Datacache and link it.
            let value_a = datacache().get_or_load_value::<T>(&self.head_seal)?;
            unsafe { *self.value_wu.get() = Arc::downgrade(&value_a) };
            value_a
        };
        Ok(value_a)
    }
}

impl<T> Clone for PlumRef<T> {
    fn clone(&self) -> Self {
        PlumRef {
            head_seal: self.head_seal.clone(),
            value_wu: std::cell::UnsafeCell::new(unsafe { &*self.value_wu.get() }.clone()),
            phantom: Default::default(),
        }
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
