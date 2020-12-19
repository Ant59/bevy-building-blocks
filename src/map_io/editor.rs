use crate::{
    map_io::{EditBuffer, ThreadLocalVoxelCache},
    Voxel, VoxelMap,
};
use bevy::ecs::{prelude::*, SystemParam};
use building_blocks::prelude::*;

/// A `SystemParam` that double-buffers writes to the `VoxelMap` and detects which chunks are
/// changed each frame. On the subsequent frame, the set of dirty and edited chunk keys will be
/// available in the `DirtyChunks` resource.
#[derive(SystemParam)]
pub struct VoxelEditor<'a, V: Voxel> {
    pub map: Res<'a, VoxelMap<V>>,
    pub local_cache: Res<'a, ThreadLocalVoxelCache<V>>,
    edit_buffer: ResMut<'a, EditBuffer<V>>,
}

impl<'a, V> VoxelEditor<'a, V>
where
    V: Voxel,
{
    /// Run `edit_func` on all voxels in `extent`. Does not mark the neighbors of edited chunks.
    pub fn edit_extent(&mut self, extent: Extent3i, edit_func: impl FnMut(Point3i, &mut V)) {
        self._edit_extent(false, extent, edit_func);
    }

    /// Run `edit_func` on all voxels in `extent`. All edited chunks and their neighbors will be
    /// marked as dirty.
    pub fn edit_extent_and_touch_neighbors(
        &mut self,
        extent: Extent3i,
        edit_func: impl FnMut(Point3i, &mut V),
    ) {
        self._edit_extent(true, extent, edit_func);
    }

    fn _edit_extent(
        &mut self,
        touch_neighbors: bool,
        extent: Extent3i,
        edit_func: impl FnMut(Point3i, &mut V),
    ) {
        let tls = self.local_cache.get();
        let reader = self.map.reader(&tls);
        self.edit_buffer
            .edit_voxels_out_of_place(&reader, extent, edit_func, touch_neighbors);
    }

    pub fn insert_chunk_and_touch_neighbors(&mut self, chunk_key: Point3i, chunk: Array3<V>) {
        self.edit_buffer.insert_chunk(true, chunk_key, chunk);
    }

    pub fn insert_chunk(&mut self, chunk_key: Point3i, chunk: Array3<V>) {
        self.edit_buffer.insert_chunk(false, chunk_key, chunk);
    }
}
