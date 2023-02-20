use crate::{Datahost, PlumURI};
use anyhow::Result;
use async_lock::RwLock;
use idp_proto::PlumHeadSeal;
use std::{any::Any, collections::HashMap, sync::Arc};

type UntypedValueGuts = dyn Any + Send + Sync;

/// This type simply encapsulates the storage of Arc<T> for arbitrary T within a Box, and the
/// casting of it to &Arc<T>.
#[derive(Debug)]
struct UntypedValue(Box<UntypedValueGuts>);

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

// TEMP HACK -- TODO: Probably can get rid of these, they should be derived automatically
unsafe impl Send for Datacache {}
unsafe impl Sync for Datacache {}

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
    pub async fn get_or_load_value<T>(&self, plum_uri: &PlumURI) -> Result<Arc<T>>
    where
        T: Any + serde::de::DeserializeOwned + Send + Sized + Sync,
    {
        if let Some(cached_value) = self
            .cached_value_mla
            .read()
            .await
            .get(plum_uri.get_plum_head_seal())
        {
            let value_a = cached_value.typed_value::<T>().clone();
            log::trace!(
                "Datacache::get_or_load_value({})\n\tcontent was already cached",
                plum_uri,
            );
            // It already exists in the cached content, so just return that.
            return Ok(value_a);
        }

        log::trace!(
            "Datacache::get_or_load_value({})\n\tcontent was not already cached; attempting to load...",
            plum_uri,
        );
        // Otherwise try to load it from Datahost.  TODO: Could load only the PlumBody, and it would be useful
        // to have a function that loads a PlumBody using a PlumHeadSeal.
        let plum_o = self
            .datahost_la
            .read()
            .await
            .load_option_plum(plum_uri.get_plum_head_seal())
            .await?;
        let plum = match plum_o {
            Some(plum) => plum,
            None => {
                // It's not present in this Datahost, so attempt to retrieve it from the remote
                // (only if the PlumURI is PlumURI::Remote).
                match plum_uri {
                    PlumURI::Local(_) => {
                        anyhow::bail!("can't load {} -- this Plum is not present on local Datahost, and this URI doesn't specify a remote from which to retrieve it", plum_uri);
                    }
                    #[allow(unused_variables)]
                    PlumURI::Remote(plum_uri_remote) => {
                        #[cfg(feature = "client")]
                        {
                            let mut idp_client = crate::IDPClient::connect(
                                plum_uri_remote.remote_server_url(),
                                self.datahost_la.clone(),
                            )
                            .await?;
                            idp_client.pull(&plum_uri_remote.plum_head_seal).await?;
                            self.datahost_la
                                .read()
                                .await
                                .load_plum(plum_uri.get_plum_head_seal())
                                .await?
                        }
                        #[cfg(not(feature = "client"))]
                        {
                            anyhow::bail!("PlumRef can't load {} -- this Plum is not present on local Datahost, and idp_core feature 'client' is not enabled, so pulling from the remote is not possible", plum_uri_remote);
                        }
                    }
                }
            }
        };

        // Now attempt to deserialize the Plum body content into the requested type.
        // NOTE/TODO: This is assuming all plum bodies are serialized via rmp_serde, which will
        // not be the case forever.
        log::trace!(
            "Datacache::get_or_load_value({})\n\tloaded plum: {:?}",
            plum_uri,
            plum,
        );
        let value: T = rmp_serde::from_read(plum.plum_body.plum_body_content.as_slice())?;
        log::trace!(
            "Datacache::get_or_load_value({})\n\tdeserialized to typed_content",
            plum_uri,
        );
        // Store the typed content
        let value_a = Arc::new(value);
        // Store it in the cache.
        self.cached_value_mla.write().await.insert(
            plum_uri.get_plum_head_seal().clone(),
            UntypedValue(Box::new(value_a.clone())),
        );
        log::trace!(
            "Datacache::get_or_load_value({})\n\tstored content in cache.",
            plum_uri,
        );

        Ok(value_a)
    }
    /// Clears the entire cache.
    pub async fn clear_cache(&self) {
        self.cached_value_mla.write().await.clear();
    }
    /// Clears a single value from the cache.  Note that this only clears the value's Arc<T> from this
    /// data structure, it doesn't clear it from any other location that's storing a clone of that Arc<T>.
    /// Thus it is possible for this call to have no effect.  Really the convention should be that if
    /// code wants to clone the Arc<T> from a PlumRef, it should create its own copy so as not to
    /// interefere with Datacache's intended operation.
    pub async fn clear_cached_value(&self, head_seal: &PlumHeadSeal) {
        self.cached_value_mla.write().await.remove(head_seal);
    }

    //
    // Datacache singleton functions -- related to PlumRef
    //

    /// Returns Some(datacache) if the Datacache singleton has been set, otherwise None.
    pub fn singleton_o() -> Option<&'static Self> {
        unsafe { DATACACHE_SINGLETON_P.as_ref() }
    }
    /// Returns Datacache singleton if it has been set, otherwise panics.
    pub fn singleton() -> &'static Self {
        Self::singleton_o().expect("programmer error: Datacache::set_singleton has not been called")
    }

    /// Sets the Datacache singleton, which is necessary to do before dereferencing any PlumRef-s.
    /// This will return the previously set Datacache singleton, if it's set.
    pub fn set_singleton(datacache_b: Box<Self>) -> Option<Box<Self>> {
        let datacache_bo = Self::take_singleton_o();
        unsafe {
            DATACACHE_SINGLETON_P = Box::into_raw(datacache_b);
        }
        datacache_bo
    }
    /// Takes back ownership of the Datacache singleton, after which it's considered un-set.  This should
    /// be called before the process ends in order to not leak memory and potentially other resources,
    /// if you care about that kind of thing.  Returns Some(datacache) if the Datacache singleton was set,
    /// otherwise None.
    pub fn take_singleton_o() -> Option<Box<Self>> {
        unsafe {
            if DATACACHE_SINGLETON_P.is_null() {
                None
            } else {
                let datacache_b = Box::from_raw(DATACACHE_SINGLETON_P);
                DATACACHE_SINGLETON_P = std::ptr::null_mut();
                Some(datacache_b)
            }
        }
    }
}

/// Pointer to the Datacache singleton.
static mut DATACACHE_SINGLETON_P: *mut Datacache = std::ptr::null_mut();
