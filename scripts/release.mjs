// Usage: pnpm release <patch|minor|major|x.y.z> [--local]
// Bumps the version everywhere, commits, tags v<version> and pushes; the Release workflow builds and publishes it.
// --local builds and signs on this machine (key at ~/.tauri/chocolatebar.key) and publishes the release with gh instead.
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const REPO = "AbdulghaniKM/ChocolateBar";
const run = (command, env) => execSync(command, { stdio: "inherit", env: { ...process.env, ...env } });
const args = process.argv.slice(2);
const local = args.includes("--local");
const bump = args.find((arg) => !arg.startsWith("--")) ?? "patch";

if (execSync("git status --porcelain").toString().trim()) {
  console.error("Working tree is not clean; commit or stash first.");
  process.exit(1);
}

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const [major, minor, patch] = pkg.version.split(".").map(Number);
const next =
  bump === "major" ? `${major + 1}.0.0`
  : bump === "minor" ? `${major}.${minor + 1}.0`
  : bump === "patch" ? `${major}.${minor}.${patch + 1}`
  : bump;
if (!/^\d+\.\d+\.\d+$/.test(next)) {
  console.error(`Invalid version "${bump}"; use patch, minor, major or x.y.z.`);
  process.exit(1);
}

pkg.version = next;
writeFileSync("package.json", JSON.stringify(pkg, null, 2) + "\n");

const replaceIn = (file, pattern, replacement) =>
  writeFileSync(file, readFileSync(file, "utf8").replace(pattern, replacement));
replaceIn("src-tauri/Cargo.toml", /^version = ".*"$/m, `version = "${next}"`);
replaceIn("src-tauri/Cargo.lock", /(name = "chocolatebar"\r?\nversion = )".*"/, `$1"${next}"`);

run("git add package.json src-tauri/Cargo.toml src-tauri/Cargo.lock");
run(`git commit -m "Release v${next}"`);

if (!local) {
  run(`git tag v${next}`);
  run("git push --follow-tags");
  console.log(`\nPushed v${next}. Track the build at https://github.com/${REPO}/actions`);
  process.exit(0);
}

const key = readFileSync(join(homedir(), ".tauri", "chocolatebar.key"), "utf8");
run("pnpm tauri build", { TAURI_SIGNING_PRIVATE_KEY: key, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: "" });

const dir = "src-tauri/target/release/bundle/nsis";
const installer = `ChocolateBar_${next}_x64-setup.exe`;
const entry = {
  signature: readFileSync(`${dir}/${installer}.sig`, "utf8").trim(),
  url: `https://github.com/${REPO}/releases/download/v${next}/${installer}`,
};
const manifest = {
  version: next,
  notes: `ChocolateBar v${next}`,
  pub_date: new Date().toISOString(),
  platforms: { "windows-x86_64": entry, "windows-x86_64-nsis": entry },
};
writeFileSync(`${dir}/latest.json`, JSON.stringify(manifest, null, 2));

run("git push");
run(
  `gh release create v${next} --target main --title "ChocolateBar v${next}" ` +
    `--notes "Download ${installer} below to install. Existing installs update themselves automatically." ` +
    `"${dir}/${installer}" "${dir}/${installer}.sig" "${dir}/latest.json"`,
);
console.log(`\nPublished https://github.com/${REPO}/releases/tag/v${next}`);
