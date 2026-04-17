# System Discovery & Validation

ระบบกวาดหาและตรวจสอบทรัพยากรอัตโนมัติ:
- **Skill Discovery**: พอร์ตมาจาก TypeScript เพื่อค้นหาเอเจนต์ในโฟลเดอร์ `agents/` และ `skills/`
- **Strict Validation**: ตรวจสอบ Schema ทันทีที่เจอไฟล์ เพื่อป้องกันข้อมูลผิดพลาดเข้าสู่ระบบ
- **Content Helpers**: จัดการการตัดแบ่ง Frontmatter, การแทนที่ $ARGUMENTS และการฉีด Path ของ Skill

**คำแนะนำสำหรับเอเจนต์:**
- ตรรกะการ Discovery ต้องเป็น Async และมีประสิทธิภาพสูง
- ห้ามโหลดไฟล์ที่ไม่ผ่านการตรวจสอบ Schema เข้าสู่ Registry หลักเด็ดขาด
