import { describe, expect, it } from "vitest";
import { detectContentSyntax } from "./detect-content-syntax";

describe("detectContentSyntax", () => {
  it("returns plaintext for empty content", () => {
    expect(detectContentSyntax("")).toBe("plaintext");
    expect(detectContentSyntax("   \n  ")).toBe("plaintext");
  });

  it("detects JSON objects and arrays", () => {
    expect(detectContentSyntax('{"a": 1}')).toBe("json");
    expect(detectContentSyntax("[1, 2, 3]")).toBe("json");
  });

  it("detects XML", () => {
    expect(detectContentSyntax('<?xml version="1.0"?><root/>')).toBe("xml");
    expect(detectContentSyntax("<note><to>User</to></note>")).toBe("xml");
  });

  it("detects markdown", () => {
    expect(detectContentSyntax("# Title\n\nBody")).toBe("markdown");
    expect(detectContentSyntax("- item one")).toBe("markdown");
    expect(detectContentSyntax("[link](https://example.com)")).toBe("markdown");
  });

  it("falls back to plaintext", () => {
    expect(detectContentSyntax("plain notes\nno special syntax")).toBe("plaintext");
  });
});
