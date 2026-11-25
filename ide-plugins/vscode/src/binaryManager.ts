import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

export function getPlatformTarget(): string {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

export function computeFileSha256(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash("sha256");
    const stream = fs.createReadStream(filePath);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", () => resolve(hash.digest("hex").toLowerCase()));
    stream.on("error", reject);
  });
}

export async function verifyFileSha256(
  filePath: string,
  expectedSha256: string
): Promise<boolean> {
  const actual = await computeFileSha256(filePath);
  return actual === expectedSha256.trim().toLowerCase();
}

export interface ResolveBinaryOptions {
  configuredPath?: string;
  extensionPath: string;
  globalStoragePath: string;
  workspaceRoots?: string[];
  pinnedChecksums?: Record<string, string>;
}

export async function resolveCoreBinaryPath(
  opts: ResolveBinaryOptions
): Promise<string> {
  const exeName = process.platform === "win32" ? "kmip-lsp.exe" : "kmip-lsp";
  const target = getPlatformTarget();

  // 1. Explicit user-configured path (air-gapped / core development)
  if (opts.configuredPath && opts.configuredPath.trim().length > 0) {
    const candidate = opts.configuredPath.trim();
    if (!fs.existsSync(candidate)) {
      throw new Error(`Configured kmip.core.binaryPath does not exist: ${candidate}`);
    }
    return candidate;
  }

  // 2. Bundled platform-specific binary inside extension
  const bundled = path.join(opts.extensionPath, "bin", target, exeName);
  if (fs.existsSync(bundled)) {
    const expected = opts.pinnedChecksums?.[target];
    if (expected && !(await verifyFileSha256(bundled, expected))) {
      throw new Error(`SHA-256 verification failed for bundled binary ${bundled}`);
    }
    return bundled;
  }

  // 3. Cached binary in globalStorage
  const cached = path.join(opts.globalStoragePath, "bin", target, exeName);
  if (fs.existsSync(cached)) {
    const expected = opts.pinnedChecksums?.[target];
    if (expected && !(await verifyFileSha256(cached, expected))) {
      throw new Error(`SHA-256 verification failed for cached binary ${cached}`);
    }
    return cached;
  }

  // 4. Local KMIPKit workspace build fallback (for contributors running Extension Host)
  for (const root of opts.workspaceRoots ?? []) {
    for (const profile of ["release", "debug"]) {
      const devBin = path.join(root, "ide-plugins", "core", "target", profile, exeName);
      if (fs.existsSync(devBin)) {
        return devBin;
      }
    }
  }

  throw new Error(
    `Could not locate kmip-lsp for ${target}. Set 'kmip.core.binaryPath' or build ide-plugins/core.`
  );
}
