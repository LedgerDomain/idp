use crate::Datahost;
use idp_proto::PlumHeadSeal;
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// This type simply encapsulates the storage of Arc<T> for arbitrary T within a Box, and the
/// casting of it to &Arc<T>.
#[derive(Debug)]
struct UntypedValue(Box<dyn Any>);

impl UntypedValue {
    fn typed_value<T: Any>(&self) -> &Arc<T> {
        self.0
            .as_ref()
            .downcast_ref::<Arc<T>>()
            .expect("programmer error: cast on an 'Any' that had a different type than Arc<T>")
    }
}

/// Datacache handles the loading, deserializing, and caching-in-memory of values as indexed
/// by their PlumHeadSeal values; a functionality used by PlumRef<T>.  Datacache is connected
/// to a single Datahost (for now; perhaps more in the future) from which it loads Plum-s.
pub struct Datacache {
    // Could conceivably have multiple datahosts, in a prioritized order.
    datahost_la: Arc<RwLock<Datahost>>,
    // TODO: Probably want to make this an LRU cache, and tracks the amount of memory per entry,
    // so that it's possible to do smart, non-total cache clearing (a kind of approximate garbage
    // collection).  TODO: Rename to cached_value_mla
    cached_value_mla: Arc<RwLock<HashMap<PlumHeadSeal, UntypedValue>>>,
}

impl Datacache {
    /// Create an empty Datacache, connected to the given Datahost.
    pub fn new(datahost_la: Arc<RwLock<Datahost>>) -> Self {
        let cached_value_mla = Arc::new(RwLock::new(HashMap::new()));
        Self {
            datahost_la,
            cached_value_mla,
        }
    }
    /// If the given value (indexed by its PlumHeadSeal) already exists in this Datacache, return
    /// it.  Otherwise load the Plum it refers to, deserialize its body as T, put that value in
    /// an Arc<T>, store a copy of that Arc<T> in this Datacache's cached values, and then return
    /// the Arc<T>.  Thus, if this function is called on an already-cached PlumHeadSeal's value,
    /// it will simply return the cached Arc<T>, thereby eliminating duplication.
    pub fn get_or_load_value<T>(&self, head_seal: &PlumHeadSeal) -> Result<Arc<T>, failure::Error>
    where
        T: Any + std::fmt::Debug + serde::de::DeserializeOwned + Sized,
    {
        if let Some(cached_value) = self.cached_value_mla.read().unwrap().get(head_seal) {
            let value_a = cached_value.typed_value::<T>().clone();
            log::trace!(
                "Datacache::load_content({:?})\n\tcontent was already cached: {:?}",
                head_seal,
                value_a,
            );
            // It already exists in the cached content, so just return that.
            return Ok(value_a);
        }

        log::trace!(
            "Datacache::load_content({:?})\n\tcontent was not already cached; attempting to load...",
            head_seal,
        );
        // Otherwise need to load it from Datahost.
        let plum = self.datahost_la.read().unwrap().load_plum(head_seal)?;
        // Now attempt to deserialize the Plum body content into the requested type.
        // NOTE/TODO: This is assuming all plum bodies are serialized via rmp_serde, which will
        // not be the case forever.
        log::trace!(
            "Datacache::load_content({:?})\n\tloaded plum: {:?}",
            head_seal,
            plum,
        );
        let value: T = rmp_serde::from_read(plum.body.body_content.as_slice())?;
        log::trace!(
            "Datacache::load_content({:?})\n\tdeserialized to typed_content: {:?}",
            head_seal,
            value,
        );
        // Store the typed content
        let value_a = Arc::new(value);
        // Store it in the cache.
        self.cached_value_mla
            .write()
            .unwrap()
            .insert(head_seal.clone(), UntypedValue(Box::new(value_a.clone())));
        log::trace!(
            "Datacache::load_content({:?})\n\tstored content in cache.",
            head_seal,
        );
        // Return it.
        Ok(value_a)
    }
    /// Clears the entire cache.
    pub fn clear_cache(&self) {
        self.cached_value_mla.write().unwrap().clear();
    }
    /// Clears a single value from the cache.  Note that this only clears the value's Arc<T> from this
    /// data structure, it doesn't clear it from any other location that's storing a clone of that Arc<T>.
    /// Thus it is possible for this call to have no effect.
    pub fn clear_cached_value(&self, head_seal: &PlumHeadSeal) {
        self.cached_value_mla.write().unwrap().remove(head_seal);
    }
}
