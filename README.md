# Bevy Building Blocks

**WARNING**: Still a work in progress. Subject to change.

Bevy plugins for the [building-blocks](https://github.com/bonsairobo/building-blocks) voxel crate:

- `MapIoPlugin`
  - Manages the `VoxelMap` resource
  - Provides the `ThreadLocalVoxelCache` resource for creating `ChunkMapReader`s
    - `ThreadLocalVoxelCache`s are flushed into the `VoxelMap`'s global cache every frame
  - Provides the `VoxelEditor` as a `SystemParam` for writing new voxels out of place
    - Edits are double-buffered and merged into the `VoxelMap` at the end of every frame
    - Modified chunk keys are tracked in the `DirtyChunks` resource for post-processing
  - Controls the size of the chunk cache by compressing LRU chunks every frame
  - Deletes any chunks marked as empty via the `EmptyChunks` resource
- `BvtPlugin`
  - Manages the `VoxelBVT` resource
  - Generates a new `OctreeSet` for each dirty chunk every frame
  - Detects empty octrees and marks the corresponding chunks for deletion
