# sim-test

Testbed for some 2d data structures, that allows for rendering of in-progress data.

## Next Steps

 - Change rendering system, wgpu seems too nitty-gritty to work with.
  - Last ditch: [write texture does not require alignment](https://docs.rs/wgpu/latest/wgpu/struct.Queue.html#method.write_texture)
 - Remake some of those maze experiments
 - Tackle AoC simulation problems


## Insane Ideas
 - Crates?
  - lock_free
  - crossbeam
 - Use async/await to allow for concurrent read/writes! Implement a custom executor that serialises updates, and then flips the buffers.
