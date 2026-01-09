# Runtime Allocator Handoff

This note tracks how the kernel transitions from early boot allocations to the long-lived physical frame allocator.

## Goals

- Preserve firmware-delivered memory topology for provenance and diagnostics.
- Protect all regions the kernel must keep reserved (identity mappings, framebuffer, carved metadata).
- Bring up allocator state before higher-level subsystems rely on dynamic memory.

## Planning Storage

`runtime_storage_plan` inspects the firmware memory map and the number of pending reservations to size the allocator’s bookkeeping arrays. It counts usable (conventional) regions, folds in reservation hints, and returns slot counts for both free runs and reserved regions. See [kernel/src/memory/allocator.rs#L32-L98](kernel/src/memory/allocator.rs#L32-L98) and [kernel/src/memory/allocator.rs#L100-L165](kernel/src/memory/allocator.rs#L100-L165).

The plan is requested in memory bring-up once identity-mapped ranges and early reservations are known. The result is logged via `debug_structured!` so the numbers are visible when debug output is enabled. Refer to [kernel/src/memory/init.rs#L228-L267](kernel/src/memory/init.rs#L228-L267).

## Carving Backing Storage

Before the runtime allocator exists, the kernel still operates with the early `FrameAllocator`. `carve_option_storage` uses that allocator to obtain physically contiguous blocks for two `Option` arrays: one tracking free `PhysFrame` runs, the other holding persistent `ReservedRegion` entries. Both buffers are zeroed and their physical spans are appended to the reservation set so they are never recycled. See [kernel/src/memory/init.rs#L268-L287](kernel/src/memory/init.rs#L268-L287).

## Initializing the Runtime Allocator

`initialize_runtime_allocator` consumes:

- A copied firmware memory map that lives in kernel-owned memory
- The full reservation list (identity ranges, framebuffer, early reservations, carved buffers)
- Mutable references to the free and reserved backing slices

It hydrates a `PhysicalAllocator`, stores it in a global cell, and makes it available through `with_runtime_allocator`. The allocator retains the original memory map, merges overlapping free runs, and enforces reservations. See [kernel/src/memory/allocator.rs#L167-L256](kernel/src/memory/allocator.rs#L167-L256).

Once initialization succeeds, memory bring-up emits `runtime allocator initialized` and immediately exercises the allocator by installing identity paging through `with_runtime_allocator`. After this point, any kernel component may obtain a mutable handle via `with_runtime_allocator` and expect consistent reservation enforcement. The transition happens in [kernel/src/memory/init.rs#L289-L313](kernel/src/memory/init.rs#L289-L313).

## Resulting Guarantees

- Every region marked during bring-up remains excluded from allocation.
- Free frame bookkeeping is sized to the observed topology, avoiding hard-coded limits.
- The allocator can be borrowed safely after `initialize_runtime_allocator` completes; callers must handle the `None` case if they run before handoff.
- Subsequent paging or allocator operations no longer depend on firmware structures.
