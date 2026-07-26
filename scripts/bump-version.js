import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const target = process.argv[2];
const newVersion = process.argv[3];

if (!target || !newVersion) {
  console.error('Usage: node scripts/bump-version.js <core|extension|desktop> <new-version>');
  process.exit(1);
}

// Basic SemVer check
if (!/^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$/.test(newVersion)) {
  console.error(`Invalid SemVer version string: "${newVersion}". Expected format like "0.1.1" or "1.0.0-rc.1".`);
  process.exit(1);
}

function updateJson(filePath, keyPath, value) {
  if (!fs.existsSync(filePath)) return;
  const content = fs.readFileSync(filePath, 'utf-8');
  const json = JSON.parse(content);
  
  let current = json;
  const keys = keyPath.split('.');
  for (let i = 0; i < keys.length - 1; i++) {
    current = current[keys[i]];
  }
  current[keys[keys.length - 1]] = value;
  
  fs.writeFileSync(filePath, `${JSON.stringify(json, null, 2)}\n`, 'utf-8');
  console.log(`Updated ${path.relative(rootDir, filePath)} (${keyPath} -> ${value})`);
}

function updateCargoToml(filePath, value) {
  if (!fs.existsSync(filePath)) return;
  let content = fs.readFileSync(filePath, 'utf-8');
  
  // Replace version under [package]
  content = content.replace(
    /(\[package\][\s\S]*?version\s*=\s*)"[^"]+"/,
    `$1"${value}"`
  );
  
  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`Updated ${path.relative(rootDir, filePath)} (package.version -> ${value})`);
}

switch (target) {
  case 'core': {
    const cargoPath = path.join(rootDir, 'vault-core', 'Cargo.toml');
    updateCargoToml(cargoPath, newVersion);
    console.log(`\nTo tag and release core:\n  git add .\n  git commit -m "chore(core): bump version to ${newVersion}"\n  git tag core-v${newVersion}\n  git push origin core-v${newVersion}\n`);
    break;
  }
  case 'extension': {
    const pkgPath = path.join(rootDir, 'vault-extension', 'package.json');
    updateJson(pkgPath, 'version', newVersion);
    console.log(`\nTo tag and release extension:\n  git add .\n  git commit -m "chore(extension): bump version to ${newVersion}"\n  git tag extension-v${newVersion}\n  git push origin extension-v${newVersion}\n`);
    break;
  }
  case 'desktop': {
    const pkgPath = path.join(rootDir, 'vault-desktop', 'package.json');
    const tauriConfPath = path.join(rootDir, 'vault-desktop', 'src-tauri', 'tauri.conf.json');
    const tauriCargoPath = path.join(rootDir, 'vault-desktop', 'src-tauri', 'Cargo.toml');
    
    updateJson(pkgPath, 'version', newVersion);
    updateJson(tauriConfPath, 'version', newVersion);
    updateCargoToml(tauriCargoPath, newVersion);
    
    console.log(`\nTo tag and release desktop:\n  git add .\n  git commit -m "chore(desktop): bump version to ${newVersion}"\n  git tag desktop-v${newVersion}\n  git push origin desktop-v${newVersion}\n`);
    break;
  }
  default: {
    console.error(`Unknown target "${target}". Expected "core", "extension", or "desktop".`);
    process.exit(1);
  }
}
