# sim-test

Testbed for some 2d data structures, that allows for rendering of in-progress data.

**Insane Idea**

Use async/await to allow for concurrent read/writes! Implement a custom executor that serialises updates, and then flips the buffers.
