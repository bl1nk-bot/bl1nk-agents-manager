use std::env;

#[derive(Debug, Clone)]
pub struct DefaultConfig {
    pub force_zsh: bool,
    pub zsh_path: String,
}

// ค่าเริ่มต้นสำหรับ config
pub const DEFAULT_CONFIG: DefaultConfig = DefaultConfig {
    force_zsh: true,  // โดยค่าเริ่มต้นเปิดใช้งาน zsh
    zsh_path: String::from("/bin/zsh"),
};

// ฟังก์ชันสำหรับตรวจสอบว่าเป็น Windows หรือไม่
fn is_windows() -> bool {
    cfg!(windows)
}

// สร้าง instance ของ config ที่ปรับตาม platform
pub fn get_platform_config() -> DefaultConfig {
    let force_zsh = if is_windows() { false } else { true };
    DefaultConfig {
        force_zsh,
        zsh_path: String::from("/bin/zsh"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_config() {
        let config = get_platform_config();
        
        // บน Windows ควรจะ force_zsh เป็น false
        // บน Unix ควรจะ force_zsh เป็น true
        if cfg!(windows) {
            assert_eq!(config.force_zsh, false);
        } else {
            assert_eq!(config.force_zsh, true);
        }
        
        assert_eq!(config.zsh_path, "/bin/zsh");
    }
}