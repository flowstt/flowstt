# Change: Update window drag behavior to allow background dragging

## Why

The application has no title bar, limiting window dragging to a small header area. Users expect to be able to drag the window by clicking anywhere on the background, similar to other minimal UI applications. This improves usability on Windows where the header drag region may not be immediately obvious.

## What Changes

- Modify the custom drag region to span the entire window background
- Interactive elements (buttons, inputs, selects, canvases, etc.) excluded from drag behavior
- Header-specific drag styling removed in favor of window-wide behavior

## Impact

- Affected specs: window-appearance
- Affected code: `src/styles.css`
