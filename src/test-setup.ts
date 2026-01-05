// Mock Tauri APIs for testing
// This file is loaded before tests run

// Mock the window.__TAURI_INTERNALS__ that Tauri uses
(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
  transformCallback: (callback: unknown) => {
    // Return a dummy callback ID
    return Math.random();
  },
  invoke: () => Promise.resolve(),
  metadata: {
    currentWindow: { label: 'main' },
    currentWebview: { label: 'main' }
  }
};
