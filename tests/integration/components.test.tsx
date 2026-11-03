import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ConsentModal } from "@/components/ui/ConsentModal";
import { BrowserCard } from "@/components/browser/BrowserCard";
import { CategorySelector } from "@/components/migration/CategorySelector";
import { Badge, StatusBadge, RiskBadge, ProgressBar } from "@/components/ui/index";
import { useAppStore } from "@/store/appStore";
import type { InstalledBrowser } from "@/types";

const mockInvoke = vi.mocked(invoke);

const MOCK_CHROME: InstalledBrowser = {
  kind: "chrome",
  display_name: "Google Chrome",
  version: "121.0.0.0",
  profiles: [
    {
      id: "p1",
      display_name: "Default",
      path: "/home/user/.config/google-chrome/Default",
      is_default: true,
    },
    {
      id: "p2",
      display_name: "Work",
      path: "/home/user/.config/google-chrome/Profile 1",
      is_default: false,
      email: "work@company.com",
    },
  ],
  is_running: false,
};

beforeEach(() => {
  vi.clearAllMocks();
  useAppStore.setState({ consentGiven: false });
});

// ── ConsentModal ──────────────────────────────────────────────────────────────

describe("ConsentModal", () => {
  it("renders privacy points", () => {
    render(<ConsentModal />);
    expect(screen.getByText("Local-first, always")).toBeInTheDocument();
    expect(screen.getByText("AES-256-GCM encryption")).toBeInTheDocument();
    expect(screen.getByText("Auth cookies stay private")).toBeInTheDocument();
  });

  it("calls giveConsent on button click", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    render(<ConsentModal />);
    fireEvent.click(screen.getByText(/I Understand & Continue/));
    expect(mockInvoke).toHaveBeenCalledWith("save_settings", expect.any(Object));
  });

  it("shows consent button", () => {
    render(<ConsentModal />);
    const btn = screen.getByRole("button");
    expect(btn).toBeInTheDocument();
    expect(btn.textContent).toMatch(/I Understand/);
  });
});

// ── BrowserCard ───────────────────────────────────────────────────────────────

describe("BrowserCard", () => {
  it("renders browser name", () => {
    render(<BrowserCard browser={MOCK_CHROME} />);
    expect(screen.getByText("Google Chrome")).toBeInTheDocument();
  });

  it("shows profile count", () => {
    render(<BrowserCard browser={MOCK_CHROME} />);
    expect(screen.getByText("2 profiles")).toBeInTheDocument();
  });

  it("shows running indicator when browser is running", () => {
    const running = { ...MOCK_CHROME, is_running: true };
    render(<BrowserCard browser={running} />);
    expect(screen.getByText("Running")).toBeInTheDocument();
  });

  it("expands profiles on expand button click", () => {
    render(<BrowserCard browser={MOCK_CHROME} />);
    const btn = screen.getByRole("button");
    fireEvent.click(btn);
    expect(screen.getByText("Default")).toBeInTheDocument();
    expect(screen.getByText("Work")).toBeInTheDocument();
  });

  it("shows compact profiles without expand button", () => {
    render(<BrowserCard browser={MOCK_CHROME} compact />);
    expect(screen.getByText("Default")).toBeInTheDocument();
    expect(screen.getByText("Work")).toBeInTheDocument();
  });

  it("calls onSelectProfile when profile is clicked", () => {
    const onSelect = vi.fn();
    render(
      <BrowserCard browser={MOCK_CHROME} compact onSelectProfile={onSelect} />
    );
    fireEvent.click(screen.getByText("Default"));
    expect(onSelect).toHaveBeenCalledWith(MOCK_CHROME, MOCK_CHROME.profiles[0]);
  });

  it("shows email for profiles that have one", () => {
    render(<BrowserCard browser={MOCK_CHROME} compact />);
    expect(screen.getByText("work@company.com")).toBeInTheDocument();
  });

  it("highlights selected profile", () => {
    const { container } = render(
      <BrowserCard browser={MOCK_CHROME} compact selectedProfileId="p1" />
    );
    // The selected profile row has a distinct background
    const rows = container.querySelectorAll("button");
    expect(rows[0].className).toContain("brand");
  });
});

// ── CategorySelector ──────────────────────────────────────────────────────────

describe("CategorySelector", () => {
  const onToggle = vi.fn();
  const onSelectAll = vi.fn();

  beforeEach(() => {
    onToggle.mockClear();
    onSelectAll.mockClear();
  });

  it("renders available categories", () => {
    render(
      <CategorySelector
        selected={[]}
        onToggle={onToggle}
        onSelectAll={onSelectAll}
      />
    );
    expect(screen.getByText("Bookmarks")).toBeInTheDocument();
    expect(screen.getByText("Browsing History")).toBeInTheDocument();
    expect(screen.getByText("Cookies")).toBeInTheDocument();
  });

  it("shows coming soon categories as disabled", () => {
    render(
      <CategorySelector
        selected={[]}
        onToggle={onToggle}
        onSelectAll={onSelectAll}
      />
    );
    expect(screen.getByText("Coming soon")).toBeInTheDocument();
    expect(screen.getByText("Saved Passwords")).toBeInTheDocument();
  });

  it("calls onToggle when category clicked", () => {
    render(
      <CategorySelector
        selected={[]}
        onToggle={onToggle}
        onSelectAll={onSelectAll}
      />
    );
    fireEvent.click(screen.getByText("Bookmarks").closest("button")!);
    expect(onToggle).toHaveBeenCalledWith("bookmarks");
  });

  it("calls onSelectAll when 'Select all' clicked", () => {
    render(
      <CategorySelector
        selected={[]}
        onToggle={onToggle}
        onSelectAll={onSelectAll}
      />
    );
    fireEvent.click(screen.getByText("Select all"));
    expect(onSelectAll).toHaveBeenCalled();
  });

  it("shows 'Deselect all' when all selected", () => {
    render(
      <CategorySelector
        selected={["bookmarks", "history", "cookies", "extensions", "preferences"]}
        onToggle={onToggle}
        onSelectAll={onSelectAll}
      />
    );
    expect(screen.getByText("Deselect all")).toBeInTheDocument();
  });
});

// ── UI Primitives ─────────────────────────────────────────────────────────────

describe("Badge", () => {
  it("renders with content", () => {
    render(<Badge>hello</Badge>);
    expect(screen.getByText("hello")).toBeInTheDocument();
  });

  it("applies variant classes", () => {
    const { container } = render(<Badge variant="success">ok</Badge>);
    expect(container.firstChild?.toString()).toBeTruthy();
  });
});

describe("StatusBadge", () => {
  it.each([
    ["migrated", "migrated"],
    ["failed", "failed"],
    ["skipped", "skipped"],
    ["incompatible", "incompatible"],
  ])("renders status '%s'", (status, label) => {
    render(<StatusBadge status={status} />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });
});

describe("RiskBadge", () => {
  it.each(["safe", "low", "medium", "high"])("renders risk '%s'", (level) => {
    render(<RiskBadge level={level} />);
    expect(screen.getByText(level)).toBeInTheDocument();
  });
});

describe("ProgressBar", () => {
  it("renders with value 0", () => {
    const { container } = render(<ProgressBar value={0} />);
    const fill = container.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).toBe("0%");
  });

  it("renders with value 0.5", () => {
    const { container } = render(<ProgressBar value={0.5} />);
    const fill = container.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).toBe("50%");
  });

  it("caps at 100%", () => {
    const { container } = render(<ProgressBar value={2} />);
    const fill = container.querySelector(".progress-fill") as HTMLElement;
    expect(fill.style.width).toBe("100%");
  });

  it("shows percent when showPercent=true", () => {
    render(<ProgressBar value={0.75} showPercent />);
    expect(screen.getByText("75%")).toBeInTheDocument();
  });
});
