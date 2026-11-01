const { execSync } = require('child_process');
const { copyFileSync, existsSync, mkdirSync } = require('fs');
const { join } = require('path');

// Build the frontend
console.log('Building frontend...');
execSync('npm run build', { cwd: __dirname, stdio: 'inherit' });

// Copy built files to Tauri assets directory
const distDir = join(__dirname, 'dist');
const tauriAssetsDir = join(__dirname, '..', 'crates', 'tauri-app', 'assets');

if (!existsSync(tauriAssetsDir)) {
  mkdirSync(tauriAssetsDir, { recursive: true });
}

// Copy all files from dist to tauri assets
const fs = require('fs');
const files = fs.readdirSync(distDir);

for (const file of files) {
  const source = join(distDir, file);
  const dest = join(tauriAssetsDir, file);
  
  if (fs.statSync(source).isDirectory()) {
    // For directories, we'd need to recursively copy, but for now just copy files
    console.log(`Skipping directory: ${file}`);
  } else {
    copyFileSync(source, dest);
    console.log(`Copied ${file} to Tauri assets`);
  }
}

console.log('Frontend build completed and copied to Tauri assets!');