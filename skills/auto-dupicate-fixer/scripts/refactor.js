#!/usr/bin/env node

/**
 * Phase 3: Refactor
 * Updates imports, removes files, merges modules
 * Supports TS/JS and Python
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const projectRoot = process.argv[2] || process.cwd();
const keepRemoveMapPath = path.join(projectRoot, '.keep-remove-map.json');
const keepRemoveMap = JSON.parse(fs.readFileSync(keepRemoveMapPath, 'utf-8'));

const refactorLog = {
  timestamp: new Date().toISOString(),
  updated_imports: [],
  deleted_files: [],
  errors: []
};

// Find all files that import from removed files
function findImportingFiles(removedFile) {
  const pattern = removedFile
    .replace(/\.(ts|tsx|js|jsx|py)$/, '')
    .replace(projectRoot, '')
    .replace(/^\//, '');
  
  try {
    const result = execSync(
      `grep -r "from.*${pattern}\\|import.*${pattern}" ${projectRoot} --include="*.ts" --include="*.tsx" --include="*.js" --include="*.jsx" --include="*.py" 2>/dev/null || true`,
      { encoding: 'utf-8' }
    );
    return result.trim().split('\n').filter(Boolean).map(line => line.split(':')[0]);
  } catch (e) {
    return [];
  }
}

// Update imports in TS/JS
function updateTsJsImports(file, oldPath, newPath) {
  try {
    let content = fs.readFileSync(file, 'utf-8');
    
    // Normalize paths
    const oldPathVariants = [
      oldPath,
      oldPath.replace(/\.(ts|tsx|js|jsx)$/, ''),
      oldPath.replace(/\/index\.(ts|tsx|js|jsx)$/, ''),
      `./${oldPath}`,
      `../${oldPath}`
    ];
    
    const newPathNormalized = newPath.replace(/\.(ts|tsx|js|jsx)$/, '');
    
    oldPathVariants.forEach(variant => {
      const regex = new RegExp(`(import|from)\\s+['"](${variant.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})['"](\\s|;)`, 'g');
      content = content.replace(regex, `$1 '${newPathNormalized}'$3`);
    });
    
    fs.writeFileSync(file, content, 'utf-8');
    refactorLog.updated_imports.push(file);
  } catch (e) {
    refactorLog.errors.push({ file, error: e.message });
  }
}

// Update imports in Python
function updatePythonImports(file, oldPath, newPath) {
  try {
    let content = fs.readFileSync(file, 'utf-8');
    
    const oldModule = oldPath.replace(/\.py$/, '').replace(/\//g, '.').replace(/\./g, '_');
    const newModule = newPath.replace(/\.py$/, '').replace(/\//g, '.').replace(/\./g, '_');
    
    // from x import y → from new_x import y
    content = content.replace(new RegExp(`from\\s+${oldModule}\\s+import`, 'g'), `from ${newModule} import`);
    
    // import x → import new_x
    content = content.replace(new RegExp(`import\\s+${oldModule}`, 'g'), `import ${newModule}`);
    
    fs.writeFileSync(file, content, 'utf-8');
    refactorLog.updated_imports.push(file);
  } catch (e) {
    refactorLog.errors.push({ file, error: e.message });
  }
}

// Main refactoring
keepRemoveMap.keep_remove_map.forEach(decision => {
  const { keep, remove } = decision;
  
  remove.forEach(removedFile => {
    console.log(`🔄 Processing removal of ${path.basename(removedFile)}`);
    
    // Find files importing removed file
    const importingFiles = findImportingFiles(removedFile);
    const isTS = removedFile.endsWith('.ts') || removedFile.endsWith('.tsx');
    const isPython = removedFile.endsWith('.py');
    
    // Update imports
    importingFiles.forEach(file => {
      if (isTS || removedFile.endsWith('.js') || removedFile.endsWith('.jsx')) {
        updateTsJsImports(file, removedFile, keep);
      } else if (isPython) {
        updatePythonImports(file, removedFile, keep);
      }
    });
    
    // Delete file
    try {
      fs.unlinkSync(removedFile);
      refactorLog.deleted_files.push(removedFile);
      console.log(`✅ Deleted ${removedFile}`);
    } catch (e) {
      refactorLog.errors.push({ file: removedFile, error: `Could not delete: ${e.message}` });
    }
  });
});

// Update barrel files (index.ts/index.js)
function updateBarrelFiles() {
  const barrelFiles = execSync(`find ${projectRoot} -name "index.ts" -o -name "index.js" -o -name "__init__.py" 2>/dev/null || true`, { encoding: 'utf-8' }).trim().split('\n').filter(Boolean);
  
  barrelFiles.forEach(barrelFile => {
    try {
      let content = fs.readFileSync(barrelFile, 'utf-8');
      let changed = false;
      
      refactorLog.deleted_files.forEach(deleted => {
        const pattern = deleted.replace(/\.(ts|tsx|js|jsx|py)$/, '').replace(/^.*\/([\w-]+)$/, '$1');
        if (content.includes(pattern)) {
          content = content.replace(new RegExp(`export.*${pattern}[^\\n]*\\n`, 'g'), '');
          changed = true;
        }
      });
      
      if (changed) {
        fs.writeFileSync(barrelFile, content, 'utf-8');
        refactorLog.updated_imports.push(barrelFile);
      }
    } catch (e) {
      console.warn(`⚠️ Could not update barrel: ${barrelFile}`);
    }
  });
}

updateBarrelFiles();

const outputFile = path.join(projectRoot, '.refactor-log.json');
fs.writeFileSync(outputFile, JSON.stringify(refactorLog, null, 2));

console.log(`\n✨ Refactor Summary:`);
console.log(`  Updated imports: ${refactorLog.updated_imports.length}`);
console.log(`  Deleted files: ${refactorLog.deleted_files.length}`);
console.log(`  Errors: ${refactorLog.errors.length}`);
console.log(`  Log: ${outputFile}`);

process.exit(refactorLog.errors.length > 0 ? 1 : 0);
