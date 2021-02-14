use super::{
    chunk_cache_flusher::chunk_cache_flusher_system,
    chunk_compressor::chunk_compressor_system,
    edit_buffer::{double_buffering_system, DirtyChunks},
    empty_chunk_remover::empty_chunk_remover_system,
    EditBuffer, EmptyChunks, ThreadLocalVoxelCache,
};

use crate::Voxel;

use bevy::{app::prelude::*, ecs::prelude::*};
use building_blocks::core::Point3i;

pub use super::chunk_compressor::ChunkCacheConfig;

/// A bevy plugin that provides dynamic read caching and compression for the `VoxelMap` resource.
///
/// This plugin expects the `VoxelMap` resource to already exist before systems are dispatched.
///
/// This plugin uses thread-local caches for voxel chunks that are decompressed during access. At
/// the end of the frame, these caches are flushed back into the `VoxelMap`'s global cache.
///
/// Constructing a cached voxel reader looks like this:
///
/// ```
/// use bevy::prelude::*;
/// use bevy_building_blocks::{bb::prelude::*, Voxel, VoxelMap, ThreadLocalVoxelCache};
///
/// fn reading_system<V: Voxel>(
///     voxel_map: Res<VoxelMap<V>>, caches: Res<ThreadLocalVoxelCache<V>>
/// ) {
///     // The TLS has to live longer than the reader.
///     let thread_local_cache = caches.get();
///     let reader = voxel_map.reader(&thread_local_cache);
///
///     let extent = Extent3i::from_min_and_shape(PointN([-100; 3]), PointN([200; 3]));
///     reader.for_each(&extent, |p: Point3i, voxel: V| {});
/// }
/// ```
///
/// If the size of the global chunk cache grows beyond a limit, one of the plugin systems will start
/// compressing the least-recently-used chunks to save space.
///
/// In order to get maximum read parallelism from the voxel map, use the `VoxelEditor`, a
/// `SystemParam` that writes your edits out of place. The edits will get merged into the `VoxelMap`
/// at the end of the same frame. The edited chunks will also be marked as "dirty" in the
/// `DirtyChunks` resource, which makes it easier to do post-processing when chunks change.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_building_blocks::{bb::prelude::*, Voxel, VoxelEditor};
///
/// fn writing_system<V: Voxel>(mut voxel_editor: VoxelEditor<V>) {
///     let extent = Extent3i::from_min_and_shape(PointN([-100; 3]), PointN([200; 3]));
///     voxel_editor.edit_extent_and_touch_neighbors(extent, |p: Point3i, voxel: &mut V| {});
/// }
/// ```
///
/// **WARNING**: Cached reads will always be flushed before double-buffered writes. This means if
/// you try to write directly into the `VoxelMap`, you risk having your changes overwritten by the
/// flush.
pub struct MapIoPlugin<V> {
    pub chunk_shape: Point3i,
    pub cache_config: ChunkCacheConfig,
    marker: std::marker::PhantomData<V>,
}

impl<V> MapIoPlugin<V> {
    pub fn new(chunk_shape: Point3i, cache_config: ChunkCacheConfig) -> Self {
        Self {
            chunk_shape,
            cache_config,
            marker: Default::default(),
        }
    }
}

impl<V> Plugin for MapIoPlugin<V>
where
    V: Voxel,
{
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(self.cache_config)
            .insert_resource(EditBuffer::<V>::new(self.chunk_shape))
            .insert_resource(DirtyChunks::default())
            .insert_resource(EmptyChunks::default())
            // Each thread gets its own local chunk cache. The local caches are flushed into the
            // global cache in the chunk_cache_flusher_system.
            .insert_resource(ThreadLocalVoxelCache::<V>::new())
            // Ordering the cache flusher and double buffering is important, because we don't want
            // to overwrite edits with locally cached chunks. Similarly, empty chunks should be
            // removed before new edits are merged in.
            .add_system_to_stage(stage::LAST, chunk_cache_flusher_system::<V>.system())
            .add_system_to_stage(stage::LAST, empty_chunk_remover_system::<V>.system())
            .add_system_to_stage(stage::LAST, double_buffering_system::<V>.system())
            .add_system_to_stage(stage::LAST, chunk_compressor_system::<V>.system());
    }
}
