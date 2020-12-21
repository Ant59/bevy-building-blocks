use super::ThreadLocalVoxelCache;

use crate::{Voxel, VoxelMap};

use bevy::prelude::*;

/// A system that flushes thread-local voxel chunk caches into the global map's cache.
pub fn chunk_cache_flusher_system<V>(
    mut local_caches: ResMut<ThreadLocalVoxelCache<V>>,
    mut voxel_map: ResMut<VoxelMap<V>>,
) where
    V: Voxel,
{
    let taken_caches = std::mem::replace(&mut *local_caches, ThreadLocalVoxelCache::new());
    for cache in taken_caches.into_iter() {
        voxel_map.voxels.storage_mut().flush_local_cache(cache);
    }
}
