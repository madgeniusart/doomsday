import { describe, it, expect } from "vitest";
import {
  formatBytes,
  truncateUrl,
  getDomainFromUrl,
  pluralize,
  groupBy,
  cn,
} from "@/utils";

describe("formatBytes", () => {
  it("formats 0 bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
  });
  it("formats kilobytes", () => {
    expect(formatBytes(1024)).toBe("1 KB");
  });
  it("formats megabytes", () => {
    expect(formatBytes(1024 * 1024)).toBe("1 MB");
  });
  it("formats gigabytes", () => {
    expect(formatBytes(1024 ** 3)).toBe("1 GB");
  });
  it("formats fractional MB", () => {
    expect(formatBytes(1536 * 1024)).toBe("1.5 GB");
  });
});

describe("truncateUrl", () => {
  it("returns short URL unchanged", () => {
    expect(truncateUrl("https://example.com", 100)).toBe("https://example.com");
  });
  it("truncates long URL to domain + path", () => {
    const url = "https://very-long-domain-name.example.com/some/very/long/path/here";
    const result = truncateUrl(url, 30);
    expect(result.length).toBeLessThanOrEqual(50); // hostname + truncated path
    expect(result).toContain("very-long-domain-name.example.com");
  });
  it("handles invalid URL", () => {
    const long = "a".repeat(80);
    expect(truncateUrl(long, 60)).toMatch(/…$/);
  });
});

describe("getDomainFromUrl", () => {
  it("extracts domain", () => {
    expect(getDomainFromUrl("https://www.example.com/path")).toBe("example.com");
  });
  it("strips www prefix", () => {
    expect(getDomainFromUrl("https://www.google.com")).toBe("google.com");
  });
  it("handles invalid URL gracefully", () => {
    expect(getDomainFromUrl("not-a-url")).toBe("not-a-url");
  });
});

describe("pluralize", () => {
  it("returns singular for 1", () => {
    expect(pluralize(1, "browser")).toBe("1 browser");
  });
  it("returns plural for 0", () => {
    expect(pluralize(0, "browser")).toBe("0 browsers");
  });
  it("returns plural for many", () => {
    expect(pluralize(5, "browser")).toBe("5 browsers");
  });
  it("uses custom plural", () => {
    expect(pluralize(2, "category", "categories")).toBe("2 categories");
  });
  it("uses custom plural for 1", () => {
    expect(pluralize(1, "category", "categories")).toBe("1 category");
  });
});

describe("groupBy", () => {
  it("groups by key function", () => {
    const items = [
      { cat: "a", val: 1 },
      { cat: "b", val: 2 },
      { cat: "a", val: 3 },
    ];
    const result = groupBy(items, (i) => i.cat);
    expect(result["a"]).toHaveLength(2);
    expect(result["b"]).toHaveLength(1);
    expect(result["a"].map((i) => i.val)).toEqual([1, 3]);
  });

  it("returns empty object for empty array", () => {
    expect(groupBy([], (i: string) => i)).toEqual({});
  });
});

describe("cn (classname merge)", () => {
  it("merges class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });
  it("handles falsy values", () => {
    expect(cn("foo", false && "bar", undefined, "baz")).toBe("foo baz");
  });
  it("merges conflicting Tailwind classes (last wins)", () => {
    expect(cn("text-red-500", "text-blue-500")).toBe("text-blue-500");
  });
});
