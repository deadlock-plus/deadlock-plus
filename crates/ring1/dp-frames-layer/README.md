# dp-frames-layer

A Vulkan implicit layer that records when the game calls `vkQueuePresentKHR`. It builds as a `cdylib` (`libdp_frames_layer.so` on Linux) and is the Linux source of frame times. It is ring 1 because it is platform code that knows nothing about the app.

## Public API

The library exports one symbol, `vkNegotiateLoaderLayerInterfaceVersion`. The Vulkan loader calls it (interface version 2) and receives `vkGetInstanceProcAddr` and `vkGetDeviceProcAddr` from it. The manifest that names it is written by `dp-frames` (`layer_install`).

## Dependencies

- `dp-frames-wire`
- Unix only: `libc`

## Behaviour

- The layer hooks `vkCreateInstance`, `vkDestroyInstance`, `vkCreateDevice`, `vkDestroyDevice` and `vkQueuePresentKHR`. It walks the loader's create-info chain, advances the link, and forwards every other call to the next layer.
- Dispatch data lives in a lock-free registry keyed by the loader's dispatch pointer. The present hook takes no lock and makes no allocation.
- The hook reads `CLOCK_MONOTONIC`, pushes the timestamp into a bounded ring and calls the next layer. A writer thread drains the ring into `<pid>.dpf`. A full ring drops frames instead of waiting. If the file cannot be written, the layer becomes a pass-through.
- The manifest enables the layer only when `DEADLOCK_PLUS_FRAMES=1` is set, and `DEADLOCK_PLUS_FRAMES_DISABLE=1` turns it off, so other Vulkan programs are never touched.
- `DEADLOCK_PLUS_FRAMES_DIR` overrides the output folder. The tests use it.

## Gotchas

- The library must be built for Linux (`x86_64-unknown-linux-gnu`). Inside Steam's runtime container it loads against the container's glibc, so build it against an old glibc when shipping.
- Only the first swapchain of a present call is recorded.
- The layer knows nothing about window focus. Time in the background shows as long frames.
- Registry nodes are never freed. Each Vulkan instance or device ever created costs a few dozen bytes.
- On other platforms the crate still compiles, with a stand-in clock, so its tests run on any host. It does nothing there.

## Testing

```
cargo test -p dp-frames-layer --lib
```

The tests drive the layer with a fake loader and fake next layer: negotiation, create-info chain handling, hooking, dispatch and teardown. They do not load a real Vulkan loader.
