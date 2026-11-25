import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import {
  getPlatformTarget,
  resolveCoreBinaryPath,
  verifyFileSha256
} from "../src/binaryManager";

describe("binaryManager", () => {
  it("verifies SHA-256 checksums accurately", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "kmip-bin-test-"));
    const binPath = path.join(tmpDir, "kmip-lsp");
    fs.writeFileSync(binPath, "test-binary-content");
    const expected = crypto.createHash("sha256").update("test-binary-content").digest("hex");

    expect(await verifyFileSha256(binPath, expected)).toBe(true);
    expect(await verifyFileSha256(binPath, "0".repeat(64))).toBe(false);
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it("rejects tampered bundled binary when pinned checksum mismatches", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "kmip-ext-test-"));
    const target = getPlatformTarget();
    const exe = process.platform === "win32" ? "kmip-lsp.exe" : "kmip-lsp";
    const bundledDir = path.join(tmpDir, "bin", target);
    fs.mkdirSync(bundledDir, { recursive: true });
    fs.writeFileSync(path.join(bundledDir, exe), "tampered");

    await expect(
      resolveCoreBinaryPath({
        extensionPath: tmpDir,
        globalStoragePath: tmpDir,
        pinnedChecksums: { [target]: "a".repeat(64) }
      })
    ).rejects.toThrow("SHA-256 verification failed");

    fs.rmSync(tmpDir, { recursive: true, force: true });
  });
});
