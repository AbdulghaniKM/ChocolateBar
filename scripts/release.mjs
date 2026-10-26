// Usage: pnpm release <patch|minor|major|x.y.z>
// Bumps the version everywhere, commits, tags v<version> and pushes; the Release workflow builds and publishes it.
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

const run = (command) => execSync(command, { stdio: "inherit" });
const bump = process.argv[2] ?? "patch";

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
run(`git tag v${next}`);
run("git push --follow-tags");
console.log(`\nPushed v${next}. Track the build at https://github.com/AbdulghaniKM/ChocolateBar/actions`);
