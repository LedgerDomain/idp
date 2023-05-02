use crate::{Datacache, PlumURI};
use anyhow::Result;
use async_lock::RwLock;
use std::{
    any::Any,
    sync::{Arc, Weak},
};

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
// TODO: Consider uniquefying the PlumHeadSeal-s (or PlumURI-s) that PlumRef uses.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlumRef<T> {
    // TODO: Consider uniquefying this (probably within Datacache) so that it's Arc<PlumURI>, and
    // therefore PlumRef is smaller overall.
    pub plum_uri: PlumURI,
    /// Weak pointer to the cached value.  The choice of Weak for this variable serves two purposes:
    /// - Allow the PlumRef to begin in a "non-linked" state (regardless of if it's cached in Datacache)
    /// - Allow the Datacache to reclaim memory if needed, by dropping the cached Arc<T>, after
    ///   which attempting to upgrade the Weak pointer will fail and cause the value to be re-cached.
    #[serde(skip)]
    value_wla: Arc<RwLock<Weak<T>>>,
}

// NOTE: Send + Sync are necessary because PlumRef::get_or_load_value returns Arc<T> which crosses await boundaries.
impl<T: Any + idp_proto::Deserializable + Send + Sync> PlumRef<T> {
    pub fn new(plum_uri: PlumURI) -> Self {
        Self {
            plum_uri,
            value_wla: Arc::new(RwLock::new(Weak::default())),
        }
    }
    pub async fn value_is_cached(&self) -> bool {
        self.value_wla.read().await.strong_count() > 0
    }
    pub async fn clear_cached_value(&self) {
        Datacache::singleton()
            .clear_cached_value(self.plum_uri.get_plum_head_seal())
            .await;
    }
    /// The only reason this returns `Result<Arc<T>, _>` instead of `Arc<T>` directly is because
    /// it calls Datacache::load_typed_content internally, which can fail because of connectivity
    /// or serialization errors.
    pub async fn value_a(&self) -> Result<Arc<T>> {
        // Attempt to upgrade the Weak<T>.  We have to assign the upgraded Weak pointer so that the
        // RwLockReadGuard is released before attempting to obtain a write lock below.
        let upgraded_value_w = self.value_wla.read().await.upgrade();
        match upgraded_value_w {
            Some(value_a) => {
                // If the Weak pointer could be upgraded (i.e. the cached value was already linked),
                // then simply return that.
                Ok(value_a)
            }
            None => {
                // Otherwise the Weak pointer is not linked, so load the content from Datacache and link it.
                let value_a = match Datacache::singleton_o() {
                    Some(datacache) => datacache.get_or_load_value::<T>(&self.plum_uri).await?,
                    None => {
                        // TODO: Handle when the Datacache singleton is not set; just fetch and load directly with
                        // no caching.  Probably print a warning as to the poor performance of this.  Perhaps
                        // another option would be to load through an external proxy which would be a less-local
                        // cache.
                        unimplemented!("For PlumRef to function, the Datacache singleton must be set using Datacache::set_singleton");
                    }
                };

                // This stores a Weak<T> pointing to value_a in this PlumRef.
                *self.value_wla.write().await = Arc::downgrade(&value_a);
                Ok(value_a)
            }
        }
    }
}

impl<T> Clone for PlumRef<T> {
    fn clone(&self) -> Self {
        PlumRef {
            plum_uri: self.plum_uri.clone(),
            value_wla: Arc::new(RwLock::new(Weak::default())),
        }
    }
}

impl<T> std::fmt::Debug for PlumRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("PlumRef")
            .field("plum_uri", &self.plum_uri)
            .finish()
    }
}

impl<T> std::fmt::Display for PlumRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "PlumRef({})", &self.plum_uri)
    }
}

impl<T: idp_proto::PlumRelational> idp_proto::PlumRelational for PlumRef<T> {
    fn accumulate_plum_relations_nonrecursive(
        &self,
        plum_relation_flags_m: &mut std::collections::HashMap<
            idp_proto::PlumHeadSeal,
            idp_proto::PlumRelationFlags,
        >,
    ) {
        // We have direct access to the PlumHeadSeal, so we can add it to the map directly.
        // TODO: Should this be a different type of relation than CONTENT_DEPENDENCY?  Maybe
        // PLUM_REF_DEPENDENCY is a worthy distinct relation type.
        plum_relation_flags_m.insert(
            self.plum_uri.get_plum_head_seal().clone(),
            idp_proto::PlumRelationFlags::CONTENT_DEPENDENCY,
        );
    }
}
