//! Bevy plugins for the building-blocks voxel library.

#[cfg(feature = "ncollide")]
mod bvt;

mod map;
mod map_io;
mod thread_local_resource;

#[cfg(feature = "ncollide")]
pub use bvt::{BVTPlugin, VoxelBVT};

pub use thread_local_resource::{ThreadLocalResource, ThreadLocalResourceHandle};

// Core data structures.
pub use map::{default_array, empty_chunk_map, VoxelMap, VoxelPalette};

// Systems and resources that facilitate voxel access.
pub use map_io::{ChunkCacheConfig, DirtyChunks, MapIoPlugin, ThreadLocalVoxelCache, VoxelEditor};

/// You can use your own type of voxel, but it must implement this trait.
pub trait Voxel: 'static + Copy + Default + Send + Sync {
    type TypeInfo: 'static + Send + Sync;

    fn get_type_index(&self) -> usize;
}

pub use building_blocks as bb;
