# Framebuffer Text Console

This note summarizes how the kernel drives the boot framebuffer for text output.

## Goals

- Provide a deterministic, firmware-independent rendering path after UEFI handoff.
- Offer a text console for diagnostics without relying on serial hardware.
- Preserve a clean abstraction boundary so higher layers only handle formatted strings.

## Surfaces and Colors

`FramebufferSurface` wraps the firmware-provided geometry (base pointer, pitch, pixel format) and validates that the reported dimensions fit the buffer. All drawing operations accept a surface and return `Err(())` if validation or bounds checks fail. See [kernel/src/framebuffer/draw.rs#L17-L66](kernel/src/framebuffer/draw.rs#L17-L66).

`FramebufferColor` is a simple RGB helper with `BLACK` and `WHITE` constants. Pixel encoding is format-aware via `encode_pixel`. Refer to [kernel/src/framebuffer/draw.rs#L5-L100](kernel/src/framebuffer/draw.rs#L5-L100).

## Drawing Primitives

Three entry points power the text console:

- `clear_black` zeroes the entire framebuffer defensively, limiting writes to the buffer size. [kernel/src/framebuffer/draw.rs#L69-L116](kernel/src/framebuffer/draw.rs#L69-L116)
- `fill_rect` fills arbitrary rectangles within bounds, used for clearing scrolled lines. [kernel/src/framebuffer/draw.rs#L118-L183](kernel/src/framebuffer/draw.rs#L118-L183)
- `draw_glyph` blits an 8×16 bitmap for a sanitized byte. Glyph bitmaps live in `font.rs` and are selected through `glyph_for`. [kernel/src/framebuffer/draw.rs#L185-L238](kernel/src/framebuffer/draw.rs#L185-L238)

## Text Console Flow

`FramebufferConsole` owns a surface, a `Viewport`, and a `Cursor`. It exposes `clear`, `write_bytes`, and implements `fmt::Write` so higher layers can stream formatted text. Construction accepts an origin offset and color, allowing future overlays (e.g., splitting the screen). See [kernel/src/framebuffer/text.rs#L14-L78](kernel/src/framebuffer/text.rs#L14-L78).

Key behaviors:

- **Sanitization:** `sanitize_byte` uppercases ASCII letters, collapses tabs to spaces, and replaces control characters with `?`. [kernel/src/framebuffer/text.rs#L10-L27](kernel/src/framebuffer/text.rs#L10-L27)
- **Wrapping:** Writes advance the cursor; when columns overflow, a newline is injected. [kernel/src/framebuffer/text.rs#L80-L134](kernel/src/framebuffer/text.rs#L80-L134)
- **Scrolling:** When the viewport fills, `scroll_up` shifts pixel rows upward and clears the bottom stripe. [kernel/src/framebuffer/text.rs#L136-L217](kernel/src/framebuffer/text.rs#L136-L217)

The viewport computes column/row counts from the framebuffer dimensions and font size. If the surface cannot host at least one glyph row and column, the console reports itself unusable, preventing accidental writes. [kernel/src/framebuffer/text.rs#L219-L267](kernel/src/framebuffer/text.rs#L219-L267)

## Integration Points

- `console::init` constructs a `FramebufferConsole` during early kernel bring-up and clears the display. [kernel/src/console/mod.rs#L73-L103](kernel/src/console/mod.rs#L73-L103)
- All console macros ultimately call `console::write`, which streams into the framebuffer console. [kernel/src/console/mod.rs#L106-L139](kernel/src/console/mod.rs#L106-L139)
- The loader uses `clear_framebuffer` for a safe initial wipe when needed. [kernel/src/framebuffer/mod.rs#L7-L15](kernel/src/framebuffer/mod.rs#L7-L15)

This module deliberately stays minimal: it assumes a linear framebuffer and fixed bitmap font, matching the project’s modern UEFI-only baseline. Future enhancements (color schemes, alternate fonts, graphical overlays) should layer atop these primitives while preserving the validated drawing contract.
