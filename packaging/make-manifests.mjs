#!/usr/bin/env node
// Generate the winget manifest set for an OpenASE-Plus release. Downloads nothing.
//
//   node packaging/make-manifests.mjs <tag> [SHA256SUMS.txt] [--date YYYY-MM-DD]
//
// SHA256SUMS.txt is the file attached to the GitHub release (default: ./SHA256SUMS.txt),
// lines of "<sha256>  <file name>". Output: packaging/winget/<version>/.
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
let date = null;
const di = args.indexOf("--date");
if (di >= 0) date = args.splice(di, 2)[1];
const [tag, sumsPath = "SHA256SUMS.txt"] = args;
if (!tag || !/^v\d+\.\d+\.\d+/.test(tag)) {
  console.error("usage: node make-manifests.mjs <tag, e.g. v0.9.1> [SHA256SUMS.txt] [--date YYYY-MM-DD]");
  process.exit(1);
}
const version = tag.slice(1);
const id = "OpenGamePlus.OpenASE-Plus";
const setupName = `OpenASE-Plus-${tag}-windows-x64-setup.exe`;
const url = `https://github.com/OpenGamePlus/OpenASE-Plus/releases/download/${tag}/${setupName}`;
// Must match AppId in packaging/windows/openase.iss (winget wants the Inno uninstall key).
const productCode = "{BE92ED7C-7E22-4C2E-A9CC-63ADB627803D}_is1";

const sums = new Map();
for (const line of readFileSync(sumsPath, "utf8").split(/\r?\n/)) {
  const m = line.match(/^([0-9a-fA-F]{64}) [ *]?(.+)$/);
  if (m) sums.set(m[2].trim(), m[1].toUpperCase());
}
const sha = sums.get(setupName);
if (!sha) {
  console.error(`${setupName} not found in ${sumsPath}`);
  process.exit(1);
}

const schema = "1.6.0";
const header = (type) =>
  `# yaml-language-server: $schema=https://aka.ms/winget-manifest.${type}.${schema}.schema.json\n\n`;
const common = `PackageIdentifier: ${id}\nPackageVersion: ${version}\n`;

const files = {
  [`${id}.yaml`]:
    header("version") + common + `DefaultLocale: en-US\nManifestType: version\nManifestVersion: ${schema}\n`,
  [`${id}.locale.en-US.yaml`]:
    header("defaultLocale") + common +
    `PackageLocale: en-US
Publisher: OpenGamePlus
PublisherUrl: https://github.com/OpenGamePlus
PublisherSupportUrl: https://github.com/OpenGamePlus/OpenASE-Plus/issues
PackageName: OpenASE-Plus
PackageUrl: https://github.com/OpenGamePlus/OpenASE-Plus
License: GPL-3.0-or-later
LicenseUrl: https://github.com/OpenGamePlus/OpenASE-Plus/blob/main/LICENSE
ShortDescription: Game server browser in the spirit of The All-Seeing Eye
Tags:
- game
- server-browser
- multiplayer
ReleaseNotesUrl: https://github.com/OpenGamePlus/OpenASE-Plus/releases/tag/${tag}
ManifestType: defaultLocale
ManifestVersion: ${schema}
`,
  [`${id}.installer.yaml`]:
    header("installer") + common +
    `InstallerType: inno
Scope: user
InstallModes:
- interactive
- silent
- silentWithProgress
UpgradeBehavior: install
Installers:
- Architecture: x64
  InstallerUrl: ${url}
  InstallerSha256: ${sha}
  ProductCode: '${productCode}'
${date ? `ReleaseDate: ${date}\n` : ""}ManifestType: installer
ManifestVersion: ${schema}
`,
};

const out = join(dirname(fileURLToPath(import.meta.url)), "winget", version);
mkdirSync(out, { recursive: true });
for (const [name, text] of Object.entries(files)) writeFileSync(join(out, name), text);
console.log(`Wrote ${Object.keys(files).length} manifests to ${out}`);
