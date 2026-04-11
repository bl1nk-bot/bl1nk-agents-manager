// src/shell_semantics.rs
//! การวิเคราะห์ความหมายของคำสั่ง Shell เพื่อใช้ในการตรวจสอบสิทธิ์
//!
//! วิเคราะห์คำสั่ง Shell อย่างง่ายเพื่อแยก "การดำเนินการของเครื่องมือเสมือน"
//! ทำให้กฎสิทธิ์ Read / Edit / Write / WebFetch / ListFiles สามารถจับคู่กับ
//! คำสั่ง Shell ที่เทียบเท่าได้ และป้องกันการหลีกเลี่ยงผ่านเครื่องมือ Shell
//!
//! # ตัวอย่าง
//!
//! ```ignore
//! extract_shell_operations("cat /etc/passwd", "/home/user")
//! // → vec![ShellOperation { virtual_tool: VirtualTool::ReadFile, file_path: "/etc/passwd" }]
//!
//! extract_shell_operations("curl https://example.com/api", "/home/user")
//! // → vec![ShellOperation { virtual_tool: VirtualTool::WebFetch, domain: "example.com" }]
//!
//! extract_shell_operations("echo hi > /etc/motd", "/home/user")
//! // → vec![ShellOperation { virtual_tool: VirtualTool::WriteFile, file_path: "/etc/motd" }]
//! ```
//!
//! # ข้อจำกัดที่ทราบ (ไม่สามารถวิเคราะห์แบบคงที่ได้)
//!
//! - การขยายตัวแปร Shell: `cat $FILE`
//! - Command substitution: `cat $(find .)`
//! - สคริปต์ตัวแปลภาษา: `python script.py`, `node x.js`
//! - เป้าหมายของ Pipe: `find . | xargs cat`
//! - นิพจน์แบบไดนามิกที่ซับซ้อน: `eval "cat $f"`

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// ประเภทข้อมูล (Types)
// ─────────────────────────────────────────────────────────────────────────────

/// เครื่องมือเสมือนที่แมปจากการดำเนินการของ Shell
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VirtualTool {
    ReadFile,
    ListDirectory,
    Edit,
    WriteFile,
    WebFetch,
    GrepSearch,
}

impl VirtualTool {
    /// แปลงเป็นสตริงชื่อเครื่องมือตามรูปแบบที่ระบบสิทธิ์ใช้
    pub fn as_str(&self) -> &'static str {
        match self {
            VirtualTool::ReadFile => "read_file",
            VirtualTool::ListDirectory => "list_directory",
            VirtualTool::Edit => "edit",
            VirtualTool::WriteFile => "write_file",
            VirtualTool::WebFetch => "web_fetch",
            VirtualTool::GrepSearch => "grep_search",
        }
    }
}

/// การดำเนินการกับไฟล์หรือเครือข่ายเสมือนที่สกัดจากคำสั่ง Shell
/// ใช้สำหรับจับคู่กับกฎสิทธิ์ Read / Edit / Write / WebFetch / ListFiles
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOperation {
    /// เครื่องมือเสมือนที่การดำเนินการนี้แมปไป
    pub virtual_tool: VirtualTool,
    /// พาธไฟล์หรือไดเรกทอรีแบบสัมบูรณ์ (สำหรับการดำเนินการกับไฟล์)
    pub file_path: Option<String>,
    /// ชื่อโดเมนโดยไม่รวมพอร์ต (สำหรับการดำเนินการ web_fetch)
    pub domain: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tokenizer (ตัวแยกคำ)
// ─────────────────────────────────────────────────────────────────────────────

/// แยกสตริงคำสั่ง Shell เป็น token โดยเคารพเครื่องหมายคำพูดเดี่ยว/คู่
/// และการ escape ด้วย backslash โดยแยกตามช่องว่างที่ไม่อยู่ในเครื่องหมายคำพูด
///
/// รับคำสั่งเดี่ยวที่ผ่านการแยก compound commands มาแล้ว
fn tokenize(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for ch in command.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && !in_single {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if !in_single && !in_double && (ch == ' ' || ch == '\t') {
            if !current.is_empty() {
                tokens.push(current);
                current = String::new();
            }
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ─────────────────────────────────────────────────────────────────────────────
// ตัวช่วยเกี่ยวกับพาธ (Path helpers)
// ─────────────────────────────────────────────────────────────────────────────

/// แปลงพาธทั้งหมดให้ใช้เครื่องหมาย `/` (forward slash) เพื่อความสอดคล้องข้ามแพลตฟอร์ม
/// และเข้ากันได้กับระบบ glob matching
fn normalize_to_posix(path: &str) -> String {
    path.replace('\\', "/")
}

/// แปลงพาธอาร์กิวเมนต์ให้เป็นพาธสัมบูรณ์แบบ POSIX
/// รองรับการขยาย `~` สำหรับ home directory และพาธสัมพัทธ์
fn resolve_path(p: &str, cwd: &str) -> String {
    let norm_p = normalize_to_posix(p);
    let norm_cwd = normalize_to_posix(cwd);

    if norm_p == "~" || norm_p.starts_with("~/") {
        if let Some(home_dir) = dirs::home_dir() {
            let home_str = normalize_to_posix(&home_dir.to_string_lossy());
            let rest = &norm_p[1..]; // "" หรือ "/some/path"
            if rest.is_empty() {
                home_str
            } else {
                format!("{}{}", home_str, rest)
            }
        } else {
            // fallback
            norm_p
        }
    } else if norm_p.starts_with('/') || (norm_p.len() > 2 && &norm_p[1..2] == ":") {
        // พาธสัมบูรณ์แบบ Unix (/foo) หรือ Windows (C:\foo)
        norm_p
    } else {
        format!("{}/{}", norm_cwd.trim_end_matches('/'), norm_p)
    }
}

/// ตรวจสอบว่า token ดูเหมือนเป็นอาร์กิวเมนต์พาธของไฟล์/ไดเรกทอรีหรือไม่
/// (ไม่ใช่ flag, ตัวแปร Shell, ตัวเลข, หรือ script expression)
fn looks_like_path(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // ตัวแปร Shell
    if s.starts_with('$') {
        return false;
    }
    // Flags
    if s.starts_with('-') {
        return false;
    }
    // ตัวเลขล้วน (เช่น count/size/mode)
    if s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // นิพจน์แบบ script (โปรแกรม awk/sed, brace expansion)
    if s.contains('{') || s.contains('}') {
        return false;
    }
    // URL จะถูกจัดการแยกโดย handler ของ web-fetch
    if s.contains("://") {
        return false;
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// การแยก redirection (Redirect extraction)
// ─────────────────────────────────────────────────────────────────────────────

struct RedirectResult {
    read_files: Vec<String>,
    write_files: Vec<String>,
}

/// แยก I/O redirections จาก array ของ token
/// แก้ไข `tokens` ในตำแหน่งเพื่อลบ operator ของ redirection และเป้าหมายของมัน
/// คืนค่าพาธสัมบูรณ์ของเป้าหมาย redirection ในรูปแบบ read / write
///
/// รองรับ:
///   `> file`   `>> file`  `< file`   (มีหรือไม่มีช่องว่าง)
///   `2> file`  `2>> file` `&> file`  `&>> file`
///   รูปแบบรวม: `>file`, `>>file`, `2>/dev/null`
fn extract_redirects(tokens: &mut Vec<String>, cwd: &str) -> RedirectResult {
    let mut read_files = Vec::new();
    let mut write_files = Vec::new();
    let mut to_remove = HashSet::new();
    let mut i = 0;

    while i < tokens.len() {
        let tok = &tokens[i];

        // ── กรณี operator แยก token ─────────────────────────────────
        if tok == ">" || tok == "1>" {
            if let Some(target) = tokens.get(i + 1) {
                if looks_like_path(target) {
                    write_files.push(resolve_path(target, cwd));
                    to_remove.insert(i);
                    to_remove.insert(i + 1);
                    i += 1;
                }
            }
        } else if tok == ">>" || tok == "1>>" {
            if let Some(target) = tokens.get(i + 1) {
                if looks_like_path(target) {
                    write_files.push(resolve_path(target, cwd));
                    to_remove.insert(i);
                    to_remove.insert(i + 1);
                    i += 1;
                }
            }
        } else if tok == "<" {
            if let Some(target) = tokens.get(i + 1) {
                if looks_like_path(target) {
                    read_files.push(resolve_path(target, cwd));
                    to_remove.insert(i);
                    to_remove.insert(i + 1);
                    i += 1;
                }
            }
        } else if tok == "2>" || tok == "2>>" || tok == "&>" || tok == "&>>" {
            // stderr / combined redirect — ใช้เป้าหมาย
            if let Some(target) = tokens.get(i + 1) {
                if target != "/dev/null" && looks_like_path(target) {
                    write_files.push(resolve_path(target, cwd));
                }
                to_remove.insert(i);
                to_remove.insert(i + 1);
                i += 1;
            }
        }
        // ── กรณี token รวมโดยไม่มีช่องว่าง: `>file`, `>>file` ฯลฯ ───
        else {
            // ใช้ regex จับรูปแบบ `^(>>|>|2>>|2>|&>>|&>|<)(.+)$`
            lazy_static! {
                static ref REDIR_COMBINED_RE: Regex =
                    Regex::new(r"^(>>|>|2>>|2>|&>>|&>|<)(.+)$").unwrap();
            }
            if let Some(caps) = REDIR_COMBINED_RE.captures(tok) {
                let op = caps.get(1).unwrap().as_str();
                let target = caps.get(2).unwrap().as_str();
                if target != "/dev/null" && looks_like_path(target) {
                    if op == "<" {
                        read_files.push(resolve_path(target, cwd));
                    } else {
                        write_files.push(resolve_path(target, cwd));
                    }
                }
                to_remove.insert(i);
            }
        }
        i += 1;
    }

    // ลบ token ที่ถูกทำเครื่องหมายออก
    let filtered: Vec<String> = tokens
        .iter()
        .enumerate()
        .filter(|(idx, _)| !to_remove.contains(idx))
        .map(|(_, s)| s.clone())
        .collect();
    *tokens = filtered;

    RedirectResult {
        read_files,
        write_files,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// การแยกอาร์กิวเมนต์ (Argument parsing)
// ─────────────────────────────────────────────────────────────────────────────

/// แยกอาร์กิวเมนต์แบบ positional (ไม่ใช่ flag) จากรายการ token
///
/// ข้าม flag ที่ขึ้นต้นด้วย `-` และ flag ที่อยู่ใน `flags_with_value`
/// จะใช้ token ถัดไปเป็นค่าของมัน
fn get_positional_args(args: &[String], flags_with_value: &HashSet<&str>) -> Vec<String> {
    let mut positional = Vec::new();
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !arg.starts_with('-') {
            positional.push(arg.clone());
            continue;
        }
        // flag: ตรวจสอบว่าใช้ token ถัดไปหรือไม่
        if flags_with_value.contains(arg.as_str()) {
            skip_next = true;
        }
        // flag ที่รวมค่ามาด้วย (`-n10`) จะถูกกรองโดย looks_like_path ในภายหลัง
    }
    positional
}

// ─────────────────────────────────────────────────────────────────────────────
// ตัวช่วยสำหรับ command handler
// ─────────────────────────────────────────────────────────────────────────────

type CommandHandler = fn(&[String], &str) -> Vec<ShellOperation>;

/// สร้าง operation แบบ read_file จากอาร์กิวเมนต์พาธแบบ positional
fn read_ops(args: &[String], cwd: &str, flags_with_value: Option<&HashSet<&str>>) -> Vec<ShellOperation> {
    let flags = flags_with_value.unwrap_or(&HashSet::new());
    get_positional_args(args, flags)
        .into_iter()
        .filter(|p| looks_like_path(p))
        .map(|p| ShellOperation {
            virtual_tool: VirtualTool::ReadFile,
            file_path: Some(resolve_path(&p, cwd)),
            domain: None,
        })
        .collect()
}

/// สร้าง operation แบบ list_directory จากอาร์กิวเมนต์พาธแบบ positional
/// ถ้าไม่มีอาร์กิวเมนต์พาธ จะใช้ cwd เป็นค่าเริ่มต้น
fn list_ops(args: &[String], cwd: &str, flags_with_value: Option<&HashSet<&str>>) -> Vec<ShellOperation> {
    let flags = flags_with_value.unwrap_or(&HashSet::new());
    let dirs: Vec<String> = get_positional_args(args, flags)
        .into_iter()
        .filter(|p| looks_like_path(p))
        .collect();
    if dirs.is_empty() {
        vec![ShellOperation {
            virtual_tool: VirtualTool::ListDirectory,
            file_path: Some(cwd.to_string()),
            domain: None,
        }]
    } else {
        dirs.into_iter()
            .map(|p| ShellOperation {
                virtual_tool: VirtualTool::ListDirectory,
                file_path: Some(resolve_path(&p, cwd)),
                domain: None,
            })
            .collect()
    }
}

/// แยกโดเมนจาก URL และคืนค่า operation แบบ web_fetch หรือ None เมื่อล้มเหลว
fn web_op(url: &str) -> Option<ShellOperation> {
    let normalized = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };
    // แยก hostname แบบง่าย (ไม่ใช้ url crate เพื่อลด dependency)
    if let Some(start) = normalized.find("://") {
        let rest = &normalized[start + 3..];
        let hostname = rest
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        if !hostname.is_empty() {
            return Some(ShellOperation {
                virtual_tool: VirtualTool::WebFetch,
                file_path: None,
                domain: Some(hostname.to_string()),
            });
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// ตารางคำสั่ง (Command dispatch table)
// ─────────────────────────────────────────────────────────────────────────────

lazy_static! {
    static ref COMMANDS: HashMap<&'static str, CommandHandler> = {
        let mut m: HashMap<&'static str, CommandHandler> = HashMap::new();

        // ฟังก์ชันช่วยสำหรับลงทะเบียน handler แบบ read
        macro_rules! read_handler {
            ($($name:expr),*) => {
                $(m.insert($name, |args, cwd| read_ops(args, cwd, None));)*
            };
        }
        // ฟังก์ชันช่วยสำหรับ handler ที่มี flags_with_value
        macro_rules! read_handler_with_flags {
            ($name:expr, $flags:expr) => {
                m.insert($name, move |args, cwd| {
                    read_ops(args, cwd, Some(&$flags))
                });
            };
        }

        // ลงทะเบียนคำสั่งอ่านไฟล์พื้นฐาน
        read_handler!("cat", "tac", "nl", "zcat", "bzcat", "xzcat", "gzcat", "lzcat",
                      "more", "most", "wc", "file", "stat", "readlink", "realpath",
                      "diff", "diff3", "sdiff", "cmp", "md5sum", "sha1sum", "sha256sum",
                      "sha512sum", "sha224sum", "sha384sum", "cksum", "b2sum", "sum",
                      "strings", "hexdump", "xxd", "od", "sort", "uniq", "cut", "paste",
                      "join", "column", "fold", "expand", "unexpand", "base64", "base32", "tr");

        // head
        let head_flags: HashSet<&str> = ["-n", "-c", "--lines", "--bytes"].iter().cloned().collect();
        read_handler_with_flags!("head", head_flags);

        // tail
        let tail_flags: HashSet<&str> = ["-n", "-c", "-s", "--lines", "--bytes", "--sleep-interval"]
            .iter().cloned().collect();
        read_handler_with_flags!("tail", tail_flags);

        // less
        let less_flags: HashSet<&str> = ["-b", "-h", "-j", "-p", "-x", "-y", "-z", "--shift", "--tabs"]
            .iter().cloned().collect();
        read_handler_with_flags!("less", less_flags);

        // file (เพิ่มเติม flags)
        let file_flags: HashSet<&str> = ["-m", "-e", "-F", "-P", "--magic-file", "--exclude", "--extension", "--separator"]
            .iter().cloned().collect();
        read_handler_with_flags!("file", file_flags);

        // stat flags
        let stat_flags: HashSet<&str> = ["-c", "-f", "--format", "--printf", "--file-system"]
            .iter().cloned().collect();
        read_handler_with_flags!("stat", stat_flags);

        // readlink flags
        let readlink_flags: HashSet<&str> = ["-e", "-f", "-m", "-q", "-s", "-v", "-z", "--canonicalize",
            "--canonicalize-existing", "--canonicalize-missing", "--no-newline", "--quiet",
            "--silent", "--verbose", "--zero"].iter().cloned().collect();
        read_handler_with_flags!("readlink", readlink_flags);

        // realpath flags
        let realpath_flags: HashSet<&str> = ["--relative-to", "--relative-base", "-e", "-m", "-s", "-z",
            "--canonicalize-existing", "--canonicalize-missing", "--logical", "--physical",
            "--no-symlinks", "--quiet", "--strip", "--zero"].iter().cloned().collect();
        read_handler_with_flags!("realpath", realpath_flags);

        // diff flags
        let diff_flags: HashSet<&str> = ["-u", "-U", "-c", "-C", "-I", "-x", "-X", "-W", "--label",
            "--to-file", "--from-file", "--width", "--horizon-lines", "--strip-trailing-cr",
            "--ignore-matching-lines", "--exclude", "--exclude-from"].iter().cloned().collect();
        read_handler_with_flags!("diff", diff_flags);

        // diff3 flags
        let diff3_flags: HashSet<&str> = ["-m", "-T", "-A", "-E", "-e", "-x", "-X", "-3", "-i", "--label"]
            .iter().cloned().collect();
        read_handler_with_flags!("diff3", diff3_flags);

        // sdiff flags
        let sdiff_flags: HashSet<&str> = ["-o", "-w", "-W", "-s", "-i", "-b", "-B", "-E", "-H"]
            .iter().cloned().collect();
        read_handler_with_flags!("sdiff", sdiff_flags);

        // cmp flags
        let cmp_flags: HashSet<&str> = ["-i", "-l", "-n", "-s", "--ignore-initial", "--bytes",
            "--print-bytes", "--quiet", "--silent", "--verbose", "--zero"].iter().cloned().collect();
        read_handler_with_flags!("cmp", cmp_flags);

        // strings flags
        let strings_flags: HashSet<&str> = ["-n", "-t", "-e", "-o", "-a", "--min-len", "--radix",
            "--encoding", "--file", "--print-file-name", "--data", "--all"].iter().cloned().collect();
        read_handler_with_flags!("strings", strings_flags);

        // hexdump flags
        let hexdump_flags: HashSet<&str> = ["-n", "-s", "-l", "-C", "-b", "-c", "-d", "-o", "-x", "-e", "-f", "-v"]
            .iter().cloned().collect();
        read_handler_with_flags!("hexdump", hexdump_flags);

        // xxd flags
        let xxd_flags: HashSet<&str> = ["-l", "-s", "-c", "-g", "-o", "-n", "-b", "-e", "-i", "-p", "-r", "-u", "-E"]
            .iter().cloned().collect();
        read_handler_with_flags!("xxd", xxd_flags);

        // od flags
        let od_flags: HashSet<&str> = ["-N", "-j", "-w", "-s", "-t", "-A", "-v", "--address-radix",
            "--endian", "--format", "--read-bytes", "--skip-bytes", "--strings",
            "--output-duplicates", "--width"].iter().cloned().collect();
        read_handler_with_flags!("od", od_flags);

        // sort flags
        let sort_flags: HashSet<&str> = ["-k", "-t", "-T", "--output", "-o", "--field-separator",
            "--key", "--temporary-directory", "--compress-program", "--batch-size", "--parallel",
            "--random-source", "--sort"].iter().cloned().collect();
        read_handler_with_flags!("sort", sort_flags);

        // uniq flags
        let uniq_flags: HashSet<&str> = ["-f", "-s", "-w", "-n", "--skip-fields", "--skip-chars", "--check-chars"]
            .iter().cloned().collect();
        read_handler_with_flags!("uniq", uniq_flags);

        // cut flags
        let cut_flags: HashSet<&str> = ["-b", "-c", "-d", "-f", "--delimiter", "--fields", "--bytes",
            "--characters", "--output-delimiter"].iter().cloned().collect();
        read_handler_with_flags!("cut", cut_flags);

        // paste flags
        let paste_flags: HashSet<&str> = ["-d", "-s", "--delimiters", "--serial"].iter().cloned().collect();
        read_handler_with_flags!("paste", paste_flags);

        // join flags
        let join_flags: HashSet<&str> = ["-t", "-1", "-2", "-j", "-o", "-a", "-e", "--field",
            "--header", "--check-order", "--nocheck-order", "--zero-terminated"]
            .iter().cloned().collect();
        read_handler_with_flags!("join", join_flags);

        // column flags
        let column_flags: HashSet<&str> = ["-t", "-s", "-n", "-c", "-o", "-x", "--table",
            "--separator", "--output-separator", "--fillrows"].iter().cloned().collect();
        read_handler_with_flags!("column", column_flags);

        // fold flags
        let fold_flags: HashSet<&str> = ["-w", "-b", "-s", "--width", "--bytes", "--spaces"]
            .iter().cloned().collect();
        read_handler_with_flags!("fold", fold_flags);

        // expand flags
        let expand_flags: HashSet<&str> = ["-t", "--tabs", "--initial"].iter().cloned().collect();
        read_handler_with_flags!("expand", expand_flags);

        // unexpand flags
        let unexpand_flags: HashSet<&str> = ["-t", "-a", "--tabs", "--all", "--first-only"]
            .iter().cloned().collect();
        read_handler_with_flags!("unexpand", unexpand_flags);

        // base64 flags
        let base64_flags: HashSet<&str> = ["-d", "-i", "-w", "--decode", "--ignore-garbage", "--wrap"]
            .iter().cloned().collect();
        read_handler_with_flags!("base64", base64_flags);
        read_handler_with_flags!("base32", base64_flags);

        // ── grep / search commands ──────────────────────────────────────────
        m.insert("grep", |args, cwd| {
            let has_pattern_flag = args.iter().any(|a| a == "-e" || a == "-f" || a.starts_with("-e") || a.starts_with("-f"));
            let is_recursive = args.iter().any(|a| ["-r", "-R", "--recursive", "--dereference-recursive"].contains(&a.as_str()));
            let flags: HashSet<&str> = ["-e", "-f", "-m", "-A", "-B", "-C", "--context", "--include",
                "--exclude", "--exclude-dir", "--max-count", "--after-context", "--before-context",
                "-n", "--line-number", "--label", "-D", "--devices", "--max-depth", "-X", "--exclude-from"]
                .iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags)
                .into_iter().filter(|p| looks_like_path(p)).collect();
            let file_paths = if has_pattern_flag { positional } else { positional.into_iter().skip(1).collect::<Vec<_>>() };
            let tool = if is_recursive { VirtualTool::ListDirectory } else { VirtualTool::ReadFile };
            file_paths.into_iter().map(|p| ShellOperation {
                virtual_tool: tool.clone(),
                file_path: Some(resolve_path(&p, cwd)),
                domain: None,
            }).collect()
        });
        m.insert("egrep", *m.get("grep").unwrap());
        m.insert("fgrep", *m.get("grep").unwrap());
        m.insert("zgrep", *m.get("grep").unwrap());
        m.insert("bzgrep", *m.get("grep").unwrap());

        // rg (ripgrep)
        m.insert("rg", |args, cwd| {
            let has_pattern_flag = args.iter().any(|a| a == "-e" || a == "-f");
            let flags: HashSet<&str> = ["-e", "-f", "-m", "-A", "-B", "-C", "-t", "-T", "-g",
                "--iglob", "--glob", "--type", "--type-not", "--max-count", "--max-depth",
                "--context", "--after-context", "--before-context", "-M", "--max-columns",
                "--field-match-separator"].iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags)
                .into_iter().filter(|p| looks_like_path(p)).collect();
            let file_paths = if has_pattern_flag { positional } else { positional.into_iter().skip(1).collect::<Vec<_>>() };
            file_paths.into_iter().map(|p| ShellOperation {
                virtual_tool: VirtualTool::ListDirectory,
                file_path: Some(resolve_path(&p, cwd)),
                domain: None,
            }).collect()
        });

        // ag (The Silver Searcher)
        m.insert("ag", |args, cwd| {
            let has_pattern_flag = args.iter().any(|a| a == "-e");
            let flags: HashSet<&str> = ["-e", "-m", "-A", "-B", "-C", "--depth", "--file-search-regex",
                "--file-search-regex-i", "--ignore", "--ignore-dir", "-n"].iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags)
                .into_iter().filter(|p| looks_like_path(p)).collect();
            let file_paths = if has_pattern_flag { positional } else { positional.into_iter().skip(1).collect::<Vec<_>>() };
            file_paths.into_iter().map(|p| ShellOperation {
                virtual_tool: VirtualTool::ListDirectory,
                file_path: Some(resolve_path(&p, cwd)),
                domain: None,
            }).collect()
        });

        // ack
        m.insert("ack", |args, cwd| {
            let flags: HashSet<&str> = ["-m", "-A", "-B", "-C", "--type", "--ignore-dir",
                "--ignore-file", "--ignore-directory", "-n"].iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags)
                .into_iter().filter(|p| looks_like_path(p)).collect();
            positional.into_iter().skip(1).map(|p| ShellOperation {
                virtual_tool: VirtualTool::ListDirectory,
                file_path: Some(resolve_path(&p, cwd)),
                domain: None,
            }).collect()
        });

        // ── Directory-listing commands ──────────────────────────────────────
        macro_rules! list_handler {
            ($($name:expr),*) => {
                $(m.insert($name, |args, cwd| list_ops(args, cwd, None));)*
            };
        }
        list_handler!("ls", "dir", "vdir");

        // exa/eza
        let exa_flags: HashSet<&str> = ["-L", "--level", "--sort", "--color", "--colour", "--group", "-I", "--ignore-glob"]
            .iter().cloned().collect();
        m.insert("exa", move |args, cwd| list_ops(args, cwd, Some(&exa_flags)));
        m.insert("eza", move |args, cwd| list_ops(args, cwd, Some(&exa_flags)));

        // lsd
        let lsd_flags: HashSet<&str> = ["--depth", "--color", "--icon", "--icon-theme", "--date",
            "--size", "--blocks", "--header", "--classic", "--no-symlink", "--ignore-glob", "-I"]
            .iter().cloned().collect();
        m.insert("lsd", move |args, cwd| list_ops(args, cwd, Some(&lsd_flags)));

        // find
        m.insert("find", |args, cwd| {
            let expression_keywords: HashSet<&str> = ["-name", "-iname", "-path", "-ipath", "-regex", "-iregex",
                "-type", "-maxdepth", "-mindepth", "-newer", "-mtime", "-atime", "-ctime", "-size",
                "-user", "-group", "-perm", "-links", "-inum", "-exec", "-execdir", "-ok", "-okdir",
                "-print", "-print0", "-ls", "-delete", "-prune", "-depth", "-empty", "-readable",
                "-writable", "-executable", "-follow", "-xdev", "-mount", "-true", "-false",
                "-not", "!", "-a", "-and", "-o", "-or"].iter().cloned().collect();
            let mut starting_points = Vec::new();
            for arg in args {
                if arg.starts_with('-') || arg == "(" || arg == ")" || expression_keywords.contains(arg.as_str()) {
                    break;
                }
                if looks_like_path(arg) {
                    starting_points.push(resolve_path(arg, cwd));
                }
            }
            if starting_points.is_empty() {
                vec![ShellOperation {
                    virtual_tool: VirtualTool::ListDirectory,
                    file_path: Some(cwd.to_string()),
                    domain: None,
                }]
            } else {
                starting_points.into_iter().map(|p| ShellOperation {
                    virtual_tool: VirtualTool::ListDirectory,
                    file_path: Some(p),
                    domain: None,
                }).collect()
            }
        });

        // tree
        let tree_flags: HashSet<&str> = ["-L", "-P", "-I", "-o", "-n", "-H", "-T", "--charset",
            "--filelimit", "--matchdirs", "--dirsfirst", "-J", "-X", "--du", "--si"]
            .iter().cloned().collect();
        m.insert("tree", move |args, cwd| list_ops(args, cwd, Some(&tree_flags)));

        // du
        let du_flags: HashSet<&str> = ["-d", "--max-depth", "--threshold", "-t", "--block-size",
            "-B", "--time-style", "--exclude", "-X", "--time", "--output"].iter().cloned().collect();
        m.insert("du", move |args, cwd| list_ops(args, cwd, Some(&du_flags)));

        // ── File-write commands ─────────────────────────────────────────────
        m.insert("touch", |args, cwd| {
            let flags: HashSet<&str> = ["-t", "-r", "--reference", "--date", "-d", "--time"]
                .iter().cloned().collect();
            get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::WriteFile,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        m.insert("mkdir", |args, cwd| {
            let flags: HashSet<&str> = ["-m", "--mode", "-Z", "--context"].iter().cloned().collect();
            get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::WriteFile,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        m.insert("mkfifo", |args, cwd| {
            let flags: HashSet<&str> = ["-m", "--mode", "-Z"].iter().cloned().collect();
            get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::WriteFile,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        m.insert("tee", |args, cwd| {
            get_positional_args(args, &HashSet::new()).into_iter().filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::WriteFile,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        // cp
        m.insert("cp", |args, cwd| {
            let flags: HashSet<&str> = ["-S", "--suffix", "-t", "--target-directory", "--backup",
                "--no-target-directory", "--sparse", "--reflink", "-Z", "--context", "--copy-contents"]
                .iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p)).collect();
            if positional.is_empty() { return vec![]; }
            if positional.len() == 1 {
                vec![ShellOperation {
                    virtual_tool: VirtualTool::ReadFile,
                    file_path: Some(resolve_path(&positional[0], cwd)),
                    domain: None,
                }]
            } else {
                let srcs = &positional[..positional.len()-1];
                let dst = &positional[positional.len()-1];
                let mut ops: Vec<ShellOperation> = srcs.iter().map(|p| ShellOperation {
                    virtual_tool: VirtualTool::ReadFile,
                    file_path: Some(resolve_path(p, cwd)),
                    domain: None,
                }).collect();
                ops.push(ShellOperation {
                    virtual_tool: VirtualTool::WriteFile,
                    file_path: Some(resolve_path(dst, cwd)),
                    domain: None,
                });
                ops
            }
        });

        // mv
        m.insert("mv", |args, cwd| {
            let flags: HashSet<&str> = ["-S", "--suffix", "-t", "--target-directory", "--backup", "-Z", "--context"]
                .iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p)).collect();
            if positional.len() < 2 { return vec![]; }
            let srcs = &positional[..positional.len()-1];
            let dst = &positional[positional.len()-1];
            let mut ops: Vec<ShellOperation> = srcs.iter().map(|p| ShellOperation {
                virtual_tool: VirtualTool::Edit,
                file_path: Some(resolve_path(p, cwd)),
                domain: None,
            }).collect();
            ops.push(ShellOperation {
                virtual_tool: VirtualTool::WriteFile,
                file_path: Some(resolve_path(dst, cwd)),
                domain: None,
            });
            ops
        });

        // install
        m.insert("install", |args, cwd| {
            let flags: HashSet<&str> = ["-m", "--mode", "-o", "--owner", "-g", "--group", "-S", "--suffix",
                "-t", "--target-directory", "-T", "--no-target-directory", "-Z", "--context", "-C", "--compare"]
                .iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p)).collect();
            if positional.len() < 2 { return vec![]; }
            let dst = &positional[positional.len()-1];
            vec![ShellOperation {
                virtual_tool: VirtualTool::WriteFile,
                file_path: Some(resolve_path(dst, cwd)),
                domain: None,
            }]
        });

        // dd
        m.insert("dd", |args, cwd| {
            let mut ops = Vec::new();
            for arg in args {
                if arg.starts_with("if=") {
                    let p = &arg[3..];
                    if looks_like_path(p) {
                        ops.push(ShellOperation {
                            virtual_tool: VirtualTool::ReadFile,
                            file_path: Some(resolve_path(p, cwd)),
                            domain: None,
                        });
                    }
                } else if arg.starts_with("of=") {
                    let p = &arg[3..];
                    if looks_like_path(p) {
                        ops.push(ShellOperation {
                            virtual_tool: VirtualTool::WriteFile,
                            file_path: Some(resolve_path(p, cwd)),
                            domain: None,
                        });
                    }
                }
            }
            ops
        });

        // ln
        m.insert("ln", |args, cwd| {
            let flags: HashSet<&str> = ["-S", "--suffix", "-t", "--target-directory", "-b", "--backup"]
                .iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p)).collect();
            if positional.len() < 2 { return vec![]; }
            let linkname = &positional[positional.len()-1];
            vec![ShellOperation {
                virtual_tool: VirtualTool::WriteFile,
                file_path: Some(resolve_path(linkname, cwd)),
                domain: None,
            }]
        });

        // ── File-edit commands ──────────────────────────────────────────────
        macro_rules! edit_handler {
            ($name:expr, $flags:expr) => {
                m.insert($name, move |args, cwd| {
                    get_positional_args(args, &$flags).into_iter().filter(|p| looks_like_path(p))
                        .map(|p| ShellOperation {
                            virtual_tool: VirtualTool::Edit,
                            file_path: Some(resolve_path(&p, cwd)),
                            domain: None,
                        }).collect()
                });
            };
        }
        let empty_flags = HashSet::new();
        edit_handler!("rm", empty_flags);
        edit_handler!("unlink", empty_flags);
        let rmdir_flags: HashSet<&str> = ["--ignore-fail-on-non-empty", "-p", "--parents"].iter().cloned().collect();
        edit_handler!("rmdir", rmdir_flags);
        let shred_flags: HashSet<&str> = ["-n", "--iterations", "-s", "--size", "--random-source"].iter().cloned().collect();
        edit_handler!("shred", shred_flags);
        let truncate_flags: HashSet<&str> = ["-s", "--size", "-r", "--reference", "-o", "-I", "-c", "--io-blocks", "--no-create"]
            .iter().cloned().collect();
        edit_handler!("truncate", truncate_flags);

        // chmod
        m.insert("chmod", |args, cwd| {
            let flags: HashSet<&str> = ["-f", "--reference", "--from"].iter().cloned().collect();
            let positional = get_positional_args(args, &flags);
            positional.into_iter().skip(1).filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::Edit,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        // chown
        m.insert("chown", |args, cwd| {
            let flags: HashSet<&str> = ["--from", "--reference"].iter().cloned().collect();
            let positional = get_positional_args(args, &flags);
            positional.into_iter().skip(1).filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::Edit,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        // chgrp
        m.insert("chgrp", |args, cwd| {
            let flags: HashSet<&str> = ["--reference"].iter().cloned().collect();
            let positional = get_positional_args(args, &flags);
            positional.into_iter().skip(1).filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::Edit,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        // rename
        m.insert("rename", |args, cwd| {
            let positional = get_positional_args(args, &HashSet::new());
            positional.into_iter().filter(|p| looks_like_path(p)).skip(2)
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::Edit,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        // sed
        m.insert("sed", |args, cwd| {
            let has_in_place = args.iter().any(|a| a == "-i" || a.starts_with("-i"));
            let has_explicit_script = args.iter().any(|a| a == "-e" || a == "-f" || a.starts_with("-e"));
            let flags: HashSet<&str> = ["-e", "-f", "--expression", "--file", "-l", "--line-length",
                "--sandbox", "-s", "--separate"].iter().cloned().collect();
            let positional: Vec<String> = get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p)).collect();
            let file_paths = if has_explicit_script { positional } else { positional.into_iter().skip(1).collect::<Vec<_>>() };
            let tool = if has_in_place { VirtualTool::Edit } else { VirtualTool::ReadFile };
            file_paths.into_iter().map(|p| ShellOperation {
                virtual_tool: tool.clone(),
                file_path: Some(resolve_path(&p, cwd)),
                domain: None,
            }).collect()
        });

        // awk
        m.insert("awk", |args, cwd| {
            let flags: HashSet<&str> = ["-F", "-f", "-v", "-m", "-W", "-M", "--source", "--include",
                "--load", "-b", "--characters-as-bytes", "-c", "--traditional", "-d", "-D",
                "--debug", "-e", "--exec", "-h", "--help", "-i", "--lint", "-o", "-p", "-r",
                "-s", "-S", "-t", "-V"].iter().cloned().collect();
            get_positional_args(args, &flags).into_iter().filter(|p| looks_like_path(p))
                .map(|p| ShellOperation {
                    virtual_tool: VirtualTool::ReadFile,
                    file_path: Some(resolve_path(&p, cwd)),
                    domain: None,
                }).collect()
        });

        // ── WebFetch commands ───────────────────────────────────────────────
        m.insert("curl", |args, _cwd| {
            let flags: HashSet<&str> = ["-o", "-O", "--output", "-u", "--user", "-A", "--user-agent",
                "-H", "--header", "-d", "--data", "--data-binary", "--data-raw", "--data-urlencode",
                "-X", "--request", "-F", "--form", "-e", "--referer", "-T", "--upload-file",
                "--cacert", "--capath", "--cert", "--key", "--pass", "-m", "--max-time",
                "--connect-timeout", "-r", "--range", "--limit-rate", "-b", "--cookie", "-c",
                "--cookie-jar", "--proxy", "-U", "--proxy-user", "-K", "--config", "--netrc-file",
                "--resolve", "--connect-to", "-w", "--write-out", "-x", "-Y", "--speed-limit",
                "--speed-time", "-y", "--max-filesize", "--proto", "--proto-redir", "-E",
                "--cert-type", "--key-type"].iter().cloned().collect();
            get_positional_args(args, &flags).into_iter()
                .filter(|p| p.contains("://") || p.starts_with("http://") || p.starts_with("https://") || p.starts_with("ftp://"))
                .filter_map(|url| web_op(&url))
                .collect()
        });

        m.insert("wget", |args, _cwd| {
            let flags: HashSet<&str> = ["-O", "--output-document", "-P", "--directory-prefix", "-o",
                "--output-file", "-a", "--append-output", "-U", "--user-agent", "--header", "-e",
                "--execute", "--tries", "-t", "-T", "--timeout", "--wait", "-w", "--quota", "-Q",
                "--bind-address", "--limit-rate", "--user", "--password", "--proxy-user",
                "--proxy-password", "-i", "--input-file", "--base", "--config", "--referer", "-D",
                "--domains", "--exclude-domains", "-I", "--include-directories", "-X",
                "--exclude-directories", "--regex-type", "-A", "-R", "--accept", "--reject",
                "--no-check-certificate", "--ca-certificate", "--ca-directory", "--certificate",
                "--private-key"].iter().cloned().collect();
            get_positional_args(args, &flags).into_iter()
                .filter(|p| p.contains("://") || p.starts_with("http://") || p.starts_with("https://"))
                .filter_map(|url| web_op(&url))
                .collect()
        });

        // fetch (BSD)
        m.insert("fetch", |args, _cwd| {
            let flags: HashSet<&str> = ["-o", "-q", "-v", "-a", "-T", "-S", "--no-verify-peer",
                "--no-verify-hostname", "--ca-cert"].iter().cloned().collect();
            get_positional_args(args, &flags).into_iter()
                .filter(|p| p.contains("://"))
                .filter_map(|url| web_op(&url))
                .collect()
        });

        m
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// คำสั่งนำหน้าแบบโปร่งใส (Transparent prefix commands)
// ─────────────────────────────────────────────────────────────────────────────

lazy_static! {
    static ref PREFIX_COMMAND_FLAGS: HashMap<&'static str, HashSet<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("sudo", ["-u", "--user", "-g", "--group", "-C", "--close-from", "-c", "--login-class",
            "-D", "--chdir", "-p", "--prompt", "-r", "--role", "-t", "--type", "-T",
            "--command-timeout", "-U", "--other-user"].iter().cloned().collect());
        m.insert("timeout", ["-s", "--signal", "-k", "--kill-after"].iter().cloned().collect());
        m
    };

    static ref PREFIX_COMMANDS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("sudo");
        s.insert("doas");
        s.insert("env");
        s.insert("time");
        s.insert("nice");
        s.insert("ionice");
        s.insert("nohup");
        s.insert("timeout");
        s.insert("unbuffer");
        s.insert("stdbuf");
        s
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// ฟังก์ชันหลัก (Main entry point)
// ─────────────────────────────────────────────────────────────────────────────

/// แยกการดำเนินการกับไฟล์/เครือข่ายเสมือนจากคำสั่ง Shell เดี่ยว
///
/// ฟังก์ชันนี้คาดหวัง **คำสั่ง Shell เดี่ยว** (ไม่มีตัวดำเนินการ `&&`, `||`, `;`, `|`)
/// ใช้ `split_compound_command()` ก่อนเรียกฟังก์ชันนี้สำหรับคำสั่งผสม
///
/// คืนค่าอาร์เรย์ว่างสำหรับ:
///   - คำสั่งที่ไม่อยู่ในตารางคำสั่งที่รู้จัก (ปลอดภัยเป็นค่าเริ่มต้น)
///   - ข้อมูลนำเข้าที่ว่างเปล่าหรือมีแต่ช่องว่าง
///   - การกำหนดค่าตัวแปรสภาพแวดล้อมล้วน ๆ (`FOO=bar`)
///
/// # พารามิเตอร์
/// - `simple_command`: คำสั่ง Shell เดี่ยวที่ไม่มีตัวดำเนินการผสม
/// - `cwd`: ไดเรกทอรีทำงานสำหรับการแปลงพาธสัมพัทธ์
pub fn extract_shell_operations(simple_command: &str, cwd: &str) -> Vec<ShellOperation> {
    if simple_command.trim().is_empty() {
        return vec![];
    }

    let mut tokens = tokenize(simple_command);
    if tokens.is_empty() {
        return vec![];
    }

    // แยก I/O redirections ก่อนส่งต่อไปยัง command handler
    // ฟังก์ชันนี้จะแก้ไข `tokens` ในตำแหน่งโดยลบ token ของ redirection ออก
    let RedirectResult { read_files, write_files } = extract_redirects(&mut tokens, cwd);

    let cmd_name = if let Some(name) = tokens.first() {
        name.as_str()
    } else {
        // มีเพียง redirection เท่านั้น (เช่น `> file` หรือ `< file`)
        let mut ops = Vec::new();
        for p in read_files {
            ops.push(ShellOperation {
                virtual_tool: VirtualTool::ReadFile,
                file_path: Some(p),
                domain: None,
            });
        }
        for p in write_files {
            ops.push(ShellOperation {
                virtual_tool: VirtualTool::WriteFile,
                file_path: Some(p),
                domain: None,
            });
        }
        return ops;
    };

    // ข้ามการกำหนดค่าตัวแปรสภาพแวดล้อมล้วน ๆ: `FOO=bar`, `FOO=bar BAR=baz`
    if cmd_name.contains('=') {
        return vec![];
    }

    let mut ops = Vec::new();

    // ── คำสั่งนำหน้าแบบโปร่งใส ───────────────────────────────────────────
    if PREFIX_COMMANDS.contains(cmd_name) {
        let flags_with_val = PREFIX_COMMAND_FLAGS.get(cmd_name);
        let mut start_idx = 1;
        while start_idx < tokens.len() {
            let t = &tokens[start_idx];
            if t.starts_with('-') {
                start_idx += 1;
                // ถ้า flag นี้ต้องการค่าอาร์กิวเมนต์แยก ให้ข้ามค่านั้นด้วย
                if let Some(flags) = flags_with_val {
                    if flags.contains(t.as_str()) && start_idx < tokens.len() && !tokens[start_idx].starts_with('-') {
                        start_idx += 1;
                    }
                }
            } else if t.contains('=') {
                // การกำหนดค่าตัวแปรสภาพแวดล้อม: ข้าม
                start_idx += 1;
            } else {
                break;
            }
        }
        // `timeout DURATION command` — duration เป็นตัวเลขที่อยู่ก่อนคำสั่งจริง
        if cmd_name == "timeout" && start_idx < tokens.len() && tokens[start_idx].chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            start_idx += 1;
        }
        if start_idx < tokens.len() {
            // สร้างคำสั่งภายในขึ้นใหม่และเรียกซ้ำ
            let inner_command = tokens[start_idx..].join(" ");
            ops.extend(extract_shell_operations(&inner_command, cwd));
        }
    } else {
        // ── ส่งต่อไปยัง handler ของคำสั่งที่รู้จัก ─────────────────────────
        if let Some(handler) = COMMANDS.get(cmd_name) {
            let args = &tokens[1..];
            ops.extend(handler(args, cwd));
        }
        // คำสั่งที่ไม่รู้จัก: คืนค่า ops ว่างเปล่า (ปลอดภัย — เราไม่เดาในสิ่งที่เราไม่รู้)
    }

    // เพิ่ม operation ที่ได้จาก redirection
    for p in read_files {
        ops.push(ShellOperation {
            virtual_tool: VirtualTool::ReadFile,
            file_path: Some(p),
            domain: None,
        });
    }
    for p in write_files {
        ops.push(ShellOperation {
            virtual_tool: VirtualTool::WriteFile,
            file_path: Some(p),
            domain: None,
        });
    }

    ops
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("cat /etc/passwd");
        assert_eq!(tokens, vec!["cat", "/etc/passwd"]);
    }

    #[test]
    fn test_tokenize_quotes() {
        let tokens = tokenize("echo \"hello world\"");
        assert_eq!(tokens, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_resolve_path() {
        let cwd = "/home/user";
        assert_eq!(resolve_path("file.txt", cwd), "/home/user/file.txt");
        assert_eq!(resolve_path("/absolute/path", cwd), "/absolute/path");
        assert!(resolve_path("~/doc", cwd).contains("/doc")); // home directory
    }

    #[test]
    fn test_extract_shell_operations_cat() {
        let ops = extract_shell_operations("cat /etc/passwd", "/home/user");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].virtual_tool, VirtualTool::ReadFile);
        assert_eq!(ops[0].file_path.as_deref(), Some("/etc/passwd"));
    }

    #[test]
    fn test_extract_shell_operations_redirect() {
        let ops = extract_shell_operations("echo hi > /tmp/out", "/home/user");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].virtual_tool, VirtualTool::WriteFile);
        assert!(ops[0].file_path.as_deref().unwrap().ends_with("/tmp/out"));
    }

    #[test]
    fn test_extract_shell_operations_curl() {
        let ops = extract_shell_operations("curl https://example.com/api", "/");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].virtual_tool, VirtualTool::WebFetch);
        assert_eq!(ops[0].domain.as_deref(), Some("example.com"));
    }

    #[test]
    fn test_extract_shell_operations_sudo() {
        let ops = extract_shell_operations("sudo cat /etc/shadow", "/");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].virtual_tool, VirtualTool::ReadFile);
        assert_eq!(ops[0].file_path.as_deref(), Some("/etc/shadow"));
    }
}