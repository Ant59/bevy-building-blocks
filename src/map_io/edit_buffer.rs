use crate::{
    map::{default_array, default_chunk_map},
    Voxel, VoxelMap,
};

use bevy::prelude::*;
use building_blocks::prelude::*;
use fnv::FnvHashSet;

/// For the sake of pipelining, all voxels edits are first written out of place here. They can later
/// be merged into another chunk map by overwriting the dirty chunks.
pub struct EditBuffer<V>
where
    V: Voxel,
{
    edited_voxels: ChunkMap3<V>,
    // Includes the edited chunks as well as their neighbors, all of which need to be re-meshed.
    dirty_chunk_keys: FnvHashSet<Point3i>,
}

impl<V> EditBuffer<V>
where
    V: Voxel,
{
    pub fn new(chunk_shape: Point3i) -> Self {
        Self {
            edited_voxels: default_chunk_map(chunk_shape),
            dirty_chunk_keys: Default::default(),
        }
    }

    /// This function does read-modify-write of the voxels in `extent`. If a chunk is missing from
    /// the backbuffer, it will be copied from the `reader` before being written.
    ///
    /// If `touch_neighbors`, then all chunks in the Moore Neighborhood of any edited chunk will be
    /// marked as dirty. This is useful when there are dependencies between adjacent chunks that
    /// must be considered during post-processing (e.g. during mesh generation).
    pub fn edit_voxels_out_of_place(
        &mut self,
        reader: &ChunkMapReader3<V>,
        extent: Extent3i,
        edit_func: impl FnMut(Point3i, &mut V),
        touch_neighbors: bool,
    ) {
        debug_assert!(reader.chunk_shape().eq(self.edited_voxels.chunk_shape()));

        // Copy any of the overlapping chunks that don't already exist in the backbuffer, i.e. those
        // chunks which haven't been modified yet.
        for chunk_key in reader.chunk_keys_for_extent(&extent) {
            self.edited_voxels.chunks.get_or_insert_with(chunk_key, || {
                reader
                    // We don't cache the chunk yet, because we're just going to modify this copy
                    // and insert back into the map later.
                    .copy_chunk_without_caching(&chunk_key)
                    .map(|c| c.as_decompressed())
                    .unwrap_or(Chunk3::with_array(default_array(
                        reader.extent_for_chunk_at_key(&chunk_key),
                    )))
            });
        }

        self.dirty_chunks_for_extent(touch_neighbors, extent);

        // Edit the backbuffer.
        self.edited_voxels.for_each_mut(&extent, edit_func);
    }

    pub fn insert_chunk(&mut self, touch_neighbors: bool, chunk_key: Point3i, chunk: Array3<V>) {
        // PERF: this could be more efficient if we just took the moore neighborhood in chunk space
        let extent = self.edited_voxels.extent_for_chunk_at_key(&chunk_key);
        self.dirty_chunks_for_extent(touch_neighbors, extent);
        self.edited_voxels
            .insert_chunk(chunk_key, Chunk3::with_array(chunk));
    }

    /// Write all of the edited chunks into `dst_map`. Returns the dirty chunks.
    pub fn merge_edits(self, dst_map: &mut ChunkMap3<V>) -> DirtyChunks {
        let EditBuffer {
            edited_voxels,
            dirty_chunk_keys,
        } = self;

        let edited_chunk_keys = edited_voxels.chunk_keys().cloned().collect();

        for (chunk_key, chunk) in edited_voxels.chunks.into_iter() {
            dst_map
                .chunks
                .insert(chunk_key, chunk.unwrap_decompressed());
        }

        DirtyChunks {
            edited_chunk_keys,
            dirty_chunk_keys,
        }
    }

    fn dirty_chunks_for_extent(&mut self, touch_neighbors: bool, extent: Extent3i) {
        // Mark the chunks and maybe their neighbors as dirty.
        let dirty_extent = if touch_neighbors {
            let chunk_shape = *self.edited_voxels.chunk_shape();

            Extent3i::from_min_and_max(extent.minimum - chunk_shape, extent.max() + chunk_shape)
        } else {
            extent
        };
        for chunk_key in self.edited_voxels.chunk_keys_for_extent(&dirty_extent) {
            self.dirty_chunk_keys.insert(chunk_key);
        }
    }
}

/// The sets of chunk keys that have either been edited directly or marked as dirty, by virtue of
/// neighboring an edited chunk.
#[derive(Default)]
pub struct DirtyChunks {
    pub edited_chunk_keys: Vec<Point3i>,
    pub dirty_chunk_keys: FnvHashSet<Point3i>,
}

/// Merges edits from the `EditBuffer` into the `VoxelMap`. By setting the `DirtyChunks` resource,
/// the `chunk_processor_system` will be notified to process dirty chunks on the next frame.
pub fn double_buffering_system<V>(
    mut voxel_map: ResMut<VoxelMap<V>>,
    mut edit_buffer: ResMut<EditBuffer<V>>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) where
    V: Voxel,
{
    let edit_buffer = std::mem::replace(
        &mut *edit_buffer,
        EditBuffer::new(*voxel_map.voxels.chunk_shape()),
    );
    *dirty_chunks = edit_buffer.merge_edits(&mut voxel_map.voxels);
}

// TODO: remove chunks when they are completely empty; maybe we could determine this with the octree
