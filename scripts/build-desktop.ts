import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "darwin") {
  throw new Error("The desktop installer currently targets macOS with Apple silicon.");
}

const root = fileURLToPath(new URL("../", import.meta.url));
const { version } = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as { version: string };
if (!/^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version)) {
  throw new Error("The app version must be a valid release version.");
}
const cargo = readFileSync(join(root, "desktop/Cargo.toml"), "utf8");
if (/^version = "([^"]+)"/m.exec(cargo)?.[1] !== version) {
  throw new Error("The website and native desktop versions must agree.");
}

function run(command: string[], env = process.env): void {
  const [executable, ...args] = command;
  if (executable === undefined) throw new Error("Missing build executable");
  const result = spawnSync(executable, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${executable} failed with exit code ${result.status}`);
}

run(["bun", "run", "desktop:assets"]);
run(["cargo", "build", "--release", "--locked", "--manifest-path", "desktop/Cargo.toml", "--target", "aarch64-apple-darwin"], {
  ...process.env,
  MACOSX_DEPLOYMENT_TARGET: "11.0",
});

const release = join(root, "desktop/target/aarch64-apple-darwin/release");
const bundle = join(release, "bundle");
const app = join(bundle, "macos/Agely.app");
const contents = join(app, "Contents");
rmSync(app, { force: true, recursive: true });
mkdirSync(join(contents, "MacOS"), { recursive: true });
mkdirSync(join(contents, "Resources"), { recursive: true });
copyFileSync(join(release, "agely"), join(contents, "MacOS/agely"));
copyFileSync(join(root, "desktop/icons/icon.icns"), join(contents, "Resources/icon.icns"));
cpSync(join(root, "desktop/licenses"), join(contents, "Resources/licenses"), { recursive: true });
copyFileSync(join(root, "desktop/assets/fonts/OFL.txt"), join(contents, "Resources/licenses/Fraunces-OFL.txt"));
copyFileSync(join(root, "LICENSE"), join(contents, "Resources/licenses/Agely-MIT.txt"));
copyFileSync(join(root, "ATTRIBUTION.md"), join(contents, "Resources/ATTRIBUTION.md"));
writeFileSync(join(contents, "Info.plist"), `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Agely</string>
  <key>CFBundleDisplayName</key><string>Agely</string>
  <key>CFBundleIdentifier</key><string>com.agely.app</string>
  <key>CFBundleExecutable</key><string>agely</string>
  <key>CFBundleIconFile</key><string>icon.icns</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${version}</string>
  <key>CFBundleVersion</key><string>${version}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.productivity</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
  <key>NSApplicationSupportsSecureRestorableState</key><true/>
</dict></plist>
`);
run(["plutil", "-lint", join(contents, "Info.plist")]);
run(["lipo", "-verify_arch", "arm64", join(contents, "MacOS/agely")]);
run(["codesign", "--force", "--options", "runtime", "--sign", "-", app]);
run(["codesign", "--verify", "--deep", "--strict", app]);

const imageName = `Agely_${version}_aarch64.dmg`;
const imageDirectory = join(bundle, "dmg");
mkdirSync(imageDirectory, { recursive: true });
const image = join(imageDirectory, imageName);
const staging = mkdtempSync(join(tmpdir(), "agely-installer-"));
try {
  run(["ditto", app, join(staging, "Agely.app")]);
  symlinkSync("/Applications", join(staging, "Applications"));
  run(["hdiutil", "create", "-ov", "-volname", "Agely", "-srcfolder", staging, "-format", "UDZO", image]);
  run(["hdiutil", "verify", image]);
} finally {
  rmSync(staging, { recursive: true, force: true });
}
const checksum = createHash("sha256").update(readFileSync(image)).digest("hex");
writeFileSync(join(imageDirectory, "SHA256SUMS.txt"), `${checksum}  ${imageName}\n`);
console.log(`Built ${app}\nInstaller ${image}\nThe developer preview uses an ad hoc signature and has no Apple notarization.`);
