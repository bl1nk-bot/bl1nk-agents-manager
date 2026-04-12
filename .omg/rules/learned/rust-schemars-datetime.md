---
name: rust-schemars-datetime
description: >
  DateTime<Utc> ของ chrono ไม่มี JsonSchema trait
  ใช้ String แทนสำหรับ schema types
globs:
  - "src/**/*.rs"
---

# Rust: schemars + DateTime Workaround

## ปัญหา
```rust
// ❌ compile error: DateTime<Utc> ไม่มี JsonSchema
#[derive(JsonSchema)]
struct MyStruct {
    timestamp: DateTime<Utc>,
}
```

## วิธีแก้
```rust
// ✅ ใช้ String แทน
#[derive(JsonSchema)]
struct MyStruct {
    timestamp: String,  // เก็บ RFC3339 string
}

// เวลาสร้าง:
timestamp: Utc::now().to_rfc3339()
```

## กฎ
- ถ้า struct ต้องมี `#[derive(JsonSchema)]` → ใช้ `String` สำหรับ timestamps
- ถ้าไม่ต้องการ JSON schema → ใช้ `DateTime<Utc>` ได้ปกติ
