use std::path::{Path, PathBuf};
use std::fs;
use crate::hooks::rules_injector::constants::{PROJECT_MARKERS, PROJECT_RULE_SUBDIRS, PROJECT_RULE_FILES, USER_RULE_DIR, RULE_EXTENSIONS, github_instructions_pattern};
use crate::hooks::rules_injector::types::RuleFileCandidate;
use std::collections::HashSet;

pub fn find_project_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_dir() {
        start_path.to_path_buf()
    } else {
        start_path.parent()?.to_path_buf()
    };

    loop {
        for marker in PROJECT_MARKERS {
            if current.join(marker).exists() {
                return Some(current);
            }
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None,
        }
    }
}

pub fn find_rule_files(
    project_root: Option<&Path>,
    home_dir: &Path,
    current_file: &Path,
) -> Vec<RuleFileCandidate> {
    let mut candidates = Vec::new();
    let mut seen_real_paths = HashSet::new();

    let mut current_dir = current_file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut distance = 0;

    // Search up to project root
    loop {
        for (parent, subdir) in PROJECT_RULE_SUBDIRS {
            let rule_dir = current_dir.join(parent).join(subdir);
            if rule_dir.is_dir() {
                let mut files = Vec::new();
                collect_rule_files_recursive(&rule_dir, &mut files);

                for file_path in files {
                    if let Ok(real_path) = fs::canonicalize(&file_path) {
                        if seen_real_paths.insert(real_path.clone()) {
                            candidates.push(RuleFileCandidate {
                                path: file_path,
                                real_path,
                                is_global: false,
                                distance,
                                is_single_file: false,
                            });
                        }
                    }
                }
            }
        }

        if let Some(root) = project_root {
            if current_dir == root { break; }
        }
        
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break,
        }
        distance += 1;
    }

    // Single-file rules at project root
    if let Some(root) = project_root {
        for rule_file in PROJECT_RULE_FILES {
            let file_path = root.join(rule_file);
            if file_path.is_file() {
                if let Ok(real_path) = fs::canonicalize(&file_path) {
                    if seen_real_paths.insert(real_path.clone()) {
                        candidates.push(RuleFileCandidate {
                            path: file_path,
                            real_path,
                            is_global: false,
                            distance: 0,
                            is_single_file: true,
                        });
                    }
                }
            }
        }
    }

    // User-level rules
    let user_rule_dir = home_dir.join(USER_RULE_DIR);
    if user_rule_dir.is_dir() {
        let mut user_files = Vec::new();
        collect_rule_files_recursive(&user_rule_dir, &mut user_files);

        for file_path in user_files {
            if let Ok(real_path) = fs::canonicalize(&file_path) {
                if seen_real_paths.insert(real_path.clone()) {
                    candidates.push(RuleFileCandidate {
                        path: file_path,
                        real_path,
                        is_global: true,
                        distance: 9999,
                        is_single_file: false,
                    });
                }
            }
        }
    }

    // Sort by distance
    candidates.sort_by(|a, b| {
        if a.is_global != b.is_global {
            a.is_global.cmp(&b.is_global)
        } else {
            a.distance.cmp(&b.distance)
        }
    });

    candidates
}

fn collect_rule_files_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rule_files_recursive(&path, results);
            } else if path.is_file() {
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let is_github_inst = dir.to_str().map_or(false, |s| s.contains(".github/instructions"));
                
                let is_valid = if is_github_inst {
                    github_instructions_pattern().is_match(filename)
                } else {
                    RULE_EXTENSIONS.iter().any(|ext| filename.ends_with(ext))
                };

                if is_valid {
                    results.push(path);
                }
            }
        }
    }
}
