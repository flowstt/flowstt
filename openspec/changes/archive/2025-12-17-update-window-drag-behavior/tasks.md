## 1. Implementation

- [x] 1.1 Add `-webkit-app-region: drag` to body element in styles.css
- [x] 1.2 Add `-webkit-app-region: no-drag` to interactive elements (buttons, inputs, selects, textareas, canvas elements, toggles)
- [x] 1.3 Remove header-specific drag cursor styling (cursor: grab/grabbing)
- [x] 1.4 Remove user-select: none from header (if only for drag purposes)

## 2. Validation

- [x] 2.1 Build application and verify window is draggable from any background area
- [x] 2.2 Verify interactive elements remain clickable (record button, device selects, toggles)
- [x] 2.3 Verify canvas elements respond to mouse events (not intercepted by drag)
