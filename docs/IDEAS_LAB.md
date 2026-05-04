# 💡 BL1NK Ideas Lab: Strategic Differentiation

ไฟล์นี้เก็บรวบรวมไอเดียจากการ Brainstorm เพื่อสร้างความแตกต่าง (Differentiation) ให้กับ bl1nk-agents โดยเน้นที่ **Tangible Results** และ **Smart Agent Management**

## 1. Tangible Results over Pure Reduction (ผลลัพธ์ที่จับต้องได้)

### 1.1 Context Durability (การยืดอายุความจำ)
- **Concept**: ไม่ใช่แค่ลด Context แต่ทำให้ "จำแม่นขึ้นโดยไม่ต้องย้ำ"
- **Implementation**: ระบบ Logic ภายนอกจะคอยสรุป Key Decision ที่เกิดขึ้นในแต่ละช่วงเวลา และ "ปักหมุด" (Pin) ไว้ในส่วนสำคัญของ Context ตลอดเวลา (Persistent Anchors)
- **Benefit**: ผู้ใช้ไม่ต้องสั่งงานซ้ำๆ และโมเดลเดิมสามารถทำงานได้นานขึ้นโดยไม่หลุดโฟกัส

### 1.2 Action-Oriented Tooling
- **Concept**: เครื่องมือต้องไม่ได้แค่ "แสดงผล" แต่ต้อง "สร้างผลลัพธ์"
- **Example**: `todo` tool จะไม่แค่ Print รายการออกมา แต่จะทำการ Sync และสร้าง/อัปเดตไฟล์ `todo.md` จริงๆ ในเครื่องผู้ใช้ทันที (Atomic Result)
- **Smart Keyword**: ตรวจจับคำสำคัญในบทสนทนาเพื่ออัปเดต Metadata หรือไฟล์โดยไม่ต้องรอคำสั่งตรงๆ

## 2. Smart Environment & Mode Awareness

### 2.1 Intent-Based Mode Switching
- **Concept**: เปลี่ยนโหมดการทำงานตามความเหมาะสมของงาน (Dynamic Modes)
- **Logic**: หากตรวจพบความคลุมเครือ (Ambiguity) หรือคำสั่งเชิงออกแบบ ระบบจะเสนอเข้าสู่ **"Brainstorm Mode"** อัตโนมัติ แทนการฝืน Implement ไปทั้งที่ข้อมูลไม่ครบ
- **Environment Sensing**: ระบบตรวจสอบเครื่องมือและสถานะปัจจุบันของผู้ใช้ (Environment Scan) เพื่อแนะนำแนวทางที่ทำได้จริง ณ ตอนนั้น

### 2.2 Hermes Adaptive Intelligence
- **Concept**: วิเคราะห์รูปแบบการทำงาน (Workflow Analysis) และความรู้สึก (Sentiment) ของผู้ใช้
- **Action**: สร้างเมมโมรี่หรือสกิลอัติโนมัติเพื่อช่วยเหลือในขั้นตอนเหล่านั้น (e.g., "ฉันเตรียมร่างอัปเดต TODO ไว้ให้แล้วหลังจากรันเทสเสร็จ")

## 3. Predictive Orchestration (AI-Augmented Tab-Completion)

### 3.1 Ghost Commands & Contextual Completion
- **Concept**: ระบบ "เดาใจ" ขั้นถัดไป (Next-Step Prediction) ทั้งสำหรับผู้ใช้และเอเจนต์
- **Implementation (User-side)**: เมื่อผู้ใช้เริ่มพิมพ์คำสั่ง ระบบจะนำประวัติงานใน `todo.md` หรือนิสัยการทำงานมาเสนอเป็น "Ghost Text" (Inline) ให้กด Tab เพื่อใช้คำสั่งนั้นทันที (เช่น พิมพ์ `imp` -> เสนอ `implement-plan phase-3-task-2`)
- **Implementation (AI-side)**: เมื่อเอเจนต์กำลังทำงานต่อเนื่อง ระบบ (Rust side) สามารถส่ง "Contextual Hints" สั้นๆ ให้เอเจนต์เพื่อลดเวลาในการคิด/วิเคราะห์ (Agent-to-Agent Tab-Completion)
- **Benefit**: ลดแรงต้าน (Friction) ในการใช้งาน สร้างความรู้สึกว่าระบบ "รู้ใจ" และ "นำหน้า" งานไปหนึ่งก้าวเสมอ

## 4. Storage & Context Hygiene

### 4.1 Smart Offloading & Pruning
- **Concept**: ไม่ใช่แค่ Archive แต่ต้องมีการ "โละ" (Pruning)
- **Logic**: ใช้กฎ 60/20/20 ในการจัดการ Context Window (60% Context / 20% System / 20% Output Buffer)
- **Action**: เริ่ม Compact ที่ 60-70% เสมอเพื่อความปลอดภัย (Stability)

---
**Last Updated**: 2026-04-20
**Status**: Open for Incubation
