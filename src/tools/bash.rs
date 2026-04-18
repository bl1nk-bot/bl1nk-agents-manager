//! เครื่องมือ Bash (Bash Tool)
//!
//! เครื่องมือสำหรับรันคำสั่งเชลล์ (Shell Commands)

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// ข้อมูลนำเข้าสำหรับเครื่องมือ bash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashInput {
    /// คำสั่งเชลล์ที่ต้องการรัน
    pub command: String,
    /// เวลาหมดเวลาเป็นวินาที (ไม่บังคับ, ค่าเริ่มต้นคือไม่จำกัดเวลา)
    pub timeout: Option<u64>,
    /// ไดเรกทอรีทำงานสำหรับคำสั่ง (ไม่บังคับ)
    pub workdir: Option<String>,
}

/// ข้อมูลส่งออกหลังจากรันเครื่องมือ bash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashOutput {
    /// ผลลัพธ์มาตรฐาน (stdout) จากคำสั่ง
    pub stdout: String,
    /// ข้อความแสดงข้อผิดพลาดมาตรฐาน (stderr) จากคำสั่ง
    pub stderr: String,
    /// รหัสสถานะการจบการทำงาน (exit code) ของคำสั่ง
    pub exit_code: i32,
}

/// รันคำสั่ง bash พร้อมกำหนดเวลาหมดเวลา (ถ้ามี)
pub fn execute_bash(input: &BashInput) -> Result<BashOutput, String> {
    tracing::info!(command = %input.command, "🚀 กำลังรันคำสั่ง Bash");
    
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&input.command);
    
    // ตั้งค่าไดเรกทอรีทำงานถ้ามีการระบุ
    if let Some(ref workdir) = input.workdir {
        cmd.current_dir(workdir);
    }
    
    // จับ stdout และ stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    // จัดการเวลาหมดเวลาถ้ามีการระบุ
    if let Some(timeout) = input.timeout {
        let timeout_secs = Duration::from_secs(timeout);
        
        // เริ่มต้นกระบวนการลูก (child process)
        let mut child = cmd.spawn().map_err(|e| {
            let err_msg = format!("ไม่สามารถเริ่มคำสั่งได้: {}", e);
            tracing::error!(error = %err_msg);
            err_msg
        })?;
        
        // ตั้งค่าตัวอ่านสำหรับ stdout และ stderr
        let stdout_reader = BufReader::new(child.stdout.take().unwrap());
        let stderr_reader = BufReader::new(child.stderr.take().unwrap());
        
        // รอให้กระบวนการทำงานเสร็จสิ้นภายในเวลาที่กำหนด
        let start = std::time::Instant::now();
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();
        let mut exit_code = -1;
        let mut finished = false;
        
        while start.elapsed() < timeout_secs {
            // ตรวจสอบว่ากระบวนการจบการทำงานหรือยัง
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_code = status.code().unwrap_or(-1);
                    finished = true;
                    break;
                }
                Ok(None) => {
                    // กระบวนการยังทำงานอยู่ หยุดรอชั่วครู่
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    let err_msg = format!("เกิดข้อผิดพลาดขณะรอกระบวนการ: {}", e);
                    tracing::error!(error = %err_msg);
                    return Err(err_msg);
                }
            }
        }
        
        if !finished {
            // สั่งยกเลิกกระบวนการเนื่องจากหมดเวลา
            let _ = child.kill();
            let _ = child.wait();
            let err_msg = format!("คำสั่งหมดเวลาหลังจากผ่านไป {} วินาที", timeout);
            tracing::warn!(warning = %err_msg);
            return Err(err_msg);
        }
        
        // อ่านผลลัพธ์ที่เหลืออยู่
        for l in stdout_reader.lines().map_while(Result::ok) {
            stdout_lines.push(l);
        }
        for l in stderr_reader.lines().map_while(Result::ok) {
            stderr_lines.push(l);
        }
        
        tracing::info!(exit_code = %exit_code, "✅ รันคำสั่ง Bash สำเร็จ (มี timeout)");
        Ok(BashOutput {
            stdout: stdout_lines.join("\n"),
            stderr: stderr_lines.join("\n"),
            exit_code,
        })
    } else {
        // กรณีไม่มีกำหนดเวลาหมดเวลา - รันตามปกติ
        let output = cmd.output().map_err(|e| {
            let err_msg = format!("ไม่สามารถรันคำสั่งได้: {}", e);
            tracing::error!(error = %err_msg);
            err_msg
        })?;
        
        let exit_code = output.status.code().unwrap_or(-1);
        tracing::info!(exit_code = %exit_code, "✅ รันคำสั่ง Bash สำเร็จ");
        
        Ok(BashOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_input_serialization() {
        let input = BashInput {
            command: "echo 'hello'".to_string(),
            timeout: Some(30),
            workdir: Some("/tmp".to_string()),
        };
        
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("echo 'hello'"));
        assert!(json.contains("30"));
        assert!(json.contains("/tmp"));
    }

    #[test]
    fn test_bash_output_serialization() {
        let output = BashOutput {
            stdout: "Hello, World!".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
        };
        
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("Hello, World!"));
        assert!(json.contains("0"));
    }

    #[test]
    fn test_execute_echo() {
        let input = BashInput {
            command: "echo 'test output'".to_string(),
            timeout: Some(10),
            workdir: None,
        };
        
        let result = execute_bash(&input);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.stdout.contains("test output"));
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn test_execute_with_workdir() {
        let input = BashInput {
            command: "pwd".to_string(),
            timeout: Some(10),
            workdir: Some("/tmp".to_string()),
        };
        
        let result = execute_bash(&input);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.stdout.contains("/tmp"));
    }

    #[test]
    fn test_execute_with_timeout() {
        let input = BashInput {
            command: "sleep 0.1 && echo 'done'".to_string(),
            timeout: Some(5),
            workdir: None,
        };
        
        let result = execute_bash(&input);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.stdout.contains("done"));
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn test_execute_stderr() {
        let input = BashInput {
            command: "echo 'error message' >&2".to_string(),
            timeout: Some(10),
            workdir: None,
        };
        
        let result = execute_bash(&input);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.stderr.contains("error message"));
    }

    #[test]
    fn test_execute_failure() {
        let input = BashInput {
            command: "exit 1".to_string(),
            timeout: Some(10),
            workdir: None,
        };
        
        let result = execute_bash(&input);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let input = BashInput {
            command: "nonexistent_command_xyz".to_string(),
            timeout: Some(10),
            workdir: None,
        };
        
        let result = execute_bash(&input);
        // This should fail because the command doesn't exist
        assert!(result.is_err() || (result.as_ref().map(|o| o.exit_code != 0).unwrap_or(false)));
    }
}
