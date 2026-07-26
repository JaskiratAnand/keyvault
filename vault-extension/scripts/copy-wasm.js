/* eslint-disable no-console */
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

// Resolve paths relative to this script's directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const sourceFile = path.resolve(
  __dirname,
  '../../vault-core/pkg/vault_core_bg.wasm',
);
const destDir = path.resolve(__dirname, '../public');
const destFile = path.resolve(destDir, 'vault_core_bg.wasm');

try {
  // Ensure the destination directory exists
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
    console.log(`Created directory: ${destDir}`);
  }

  // Copy the WASM file
  fs.copyFileSync(sourceFile, destFile);
  console.log(
    `Successfully copied Wasm binary:\n  Source:      ${sourceFile}\n  Destination: ${destFile}`,
  );
} catch (error) {
  console.error(`Failed to copy WASM binary: ${error.message}`);
  process.exit(1);
}
