import "@testing-library/react";
// Registers jest-dom's matchers (toBeInTheDocument, toHaveClass, etc.) on
// Vitest's `expect` via module augmentation — required for every
// `.toBeInTheDocument()` call in the integration tests to type-check and
// to exist at runtime at all.
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Mock Tauri APIs for tests (not available in jsdom)
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
}));
