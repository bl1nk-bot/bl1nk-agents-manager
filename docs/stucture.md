ฉันจะช่วยคุณผสานรวม **Ashval Task Assistant Pro** (HTML UI) กับ **7 System Prompts สำหรับ Agent-based Workflow** ให้เป็นระบบที่ใช้ได้จริง ✅

---

## 📋 ภาพรวมการผสานรวม

```
┌─────────────────────────────────────────────────────────┐
│         Ashval Task Assistant Pro (Frontend)             │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Dashboard | Tasks | Mood | AI | Media | Gallery │   │
│  └──────────────────────────────────────────────────┘   │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│        Agent Orchestrator (Backend)                      │
│  ┌──────────────────────────────────────────────────┐   │
│  │ • Project Manager    • Architecture Designer     │   │
│  │ • OAuth Workspace    • Reviewer                  │   │
│  │ • Document Writer    • CI/CD Deployment         │   │
│  │ • Telemetry Feedback                            │   │
│  └──────────────────────────────────────────────────┘   │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│    External APIs & Services                             │
│  (Poe AI, YouTube, Spotify, Notion, GitHub Actions)     │
└─────────────────────────────────────────────────────────┘
```

---

## 🚀 ขั้นตอนการผสานรวม (5 ขั้น)

### **ขั้นที่ 1: ตั้งค่าโครงสร้าง Backend**

สร้างโฟลเดอร์ดังนี้:

````bash
project-root/
├── frontend/
│   └── index.html                    # ไฟล์ UI ที่มีอยู่
├── service/
│   ├── agents/
│   │   ├── project-manager.ts
│   │   ├── architecture-designer.ts
│   │   ├── oauth-workspace.ts
│   │   ├── reviewer.ts
│   │   ├── document-writer.ts
│   │   ├── cicd-deployment.ts
│   │   └── telemetry-feedback.ts
│   ├── orchestrator/
│   │   ├── agent-runner.ts
│   │   └── workflow-engine.ts
│   ├── api/
│   │   ├── routes.ts
│   │   └── middleware.ts
│   └── server.ts
├── docs/
│   └── agents/                       # ไฟล์ system prompts
└── package.json
````

---

### **ขั้นที่ 2: สร้าง Agent Interface (TypeScript)**

สร้างไฟล์ `service/types/agent.ts`:

````typescript
// Agent Base Interface
export interface Agent {
  name: string;
  role: string;
  systemPrompt: string;
  execute(input: AgentInput): Promise<AgentOutput>;
}

export interface AgentInput {
  type: string;
  data: Record<string, any>;
  context?: Record<string, any>;
}

export interface AgentOutput {
  success: boolean;
  data?: any;
  error?: string;
  timestamp: string;
  agentName: string;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  status: 'pending' | 'in-progress' | 'review' | 'done';
  assignedAgent: string;
  priority: 'high' | 'medium' | 'low';
  dueDate?: string;
  dependencies?: string[];
}

export interface WorkflowState {
  tasks: Task[];
  activeAgents: string[];
  completedTasks: string[];
  blockers: string[];
}
````

---

### **ขั้นที่ 3: สร้าง Agent Orchestrator**

สร้างไฟล์ `service/orchestrator/agent-runner.ts`:

````typescript
import { Agent, AgentInput, AgentOutput, Task, WorkflowState } from '../types/agent';

export class AgentOrchestrator {
  private agents: Map<string, Agent> = new Map();
  private workflowState: WorkflowState = {
    tasks: [],
    activeAgents: [],
    completedTasks: [],
    blockers: []
  };

  // ลงทะเบียน Agent
  registerAgent(agent: Agent): void {
    this.agents.set(agent.name, agent);
    console.log(`✅ Agent registered: ${agent.name}`);
  }

  // รัน Workflow
  async runWorkflow(tasks: Task[]): Promise<WorkflowState> {
    this.workflowState.tasks = tasks;

    for (const task of tasks) {
      if (task.status === 'pending') {
        await this.executeTask(task);
      }
    }

    return this.workflowState;
  }

  // รัน Task เดี่ยว
  private async executeTask(task: Task): Promise<void> {
    const agent = this.agents.get(task.assignedAgent);
    
    if (!agent) {
      this.workflowState.blockers.push(
        `❌ Agent not found: ${task.assignedAgent}`
      );
      return;
    }

    try {
      this.workflowState.activeAgents.push(agent.name);
      task.status = 'in-progress';

      const input: AgentInput = {
        type: task.title,
        data: { task },
        context: this.workflowState
      };

      const output = await agent.execute(input);

      if (output.success) {
        task.status = 'done';
        this.workflowState.completedTasks.push(task.id);
      } else {
        task.status = 'review';
        this.workflowState.blockers.push(output.error || 'Unknown error');
      }
    } catch (error) {
      task.status = 'review';
      this.workflowState.blockers.push(`Error: ${error}`);
    } finally {
      this.workflowState.activeAgents = 
        this.workflowState.activeAgents.filter(a => a !== agent.name);
    }
  }

  // ได้รับสถานะปัจจุบัน
  getStatus(): WorkflowState {
    return this.workflowState;
  }
}
````

---

### **ขั้นที่ 4: สร้าง Agent ตัวอย่าง (Project Manager)**

สร้างไฟล์ `service/agents/project-manager.ts`:

````typescript
import { Agent, AgentInput, AgentOutput } from '../types/agent';

export class ProjectManagerAgent implements Agent {
  name = 'ProjectManager';
  role = 'Project Manager Agent';
  systemPrompt = `
    You are the Project Manager Agent for bl1nk/BLinkOS.
    Your job is to:
    1. Break down requirements into tasks
    2. Assign tasks to appropriate agents
    3. Track progress
    4. Identify blockers
  `;

  async execute(input: AgentInput): Promise<AgentOutput> {
    try {
      const { task } = input.data;

      // ตัวอย่าง: แตกงานใหญ่เป็นงานย่อย
      const subtasks = this.breakDownTask(task);

      return {
        success: true,
        data: {
          originalTask: task,
          subtasks,
          assignments: this.assignTasks(subtasks)
        },
        timestamp: new Date().toISOString(),
        agentName: this.name
      };
    } catch (error) {
      return {
        success: false,
        error: String(error),
        timestamp: new Date().toISOString(),
        agentName: this.name
      };
    }
  }

  private breakDownTask(task: any): any[] {
    // ตัวอย่างการแตกงาน
    return [
      {
        id: `${task.id}-1`,
        title: `Define: ${task.title}`,
        type: 'definition',
        owner: 'Architecture'
      },
      {
        id: `${task.id}-2`,
        title: `Implement: ${task.title}`,
        type: 'implementation',
        owner: 'Reviewer'
      },
      {
        id: `${task.id}-3`,
        title: `Document: ${task.title}`,
        type: 'documentation',
        owner: 'DocumentWriter'
      }
    ];
  }

  private assignTasks(subtasks: any[]): Record<string, any> {
    const assignments: Record<string, any> = {};
    
    subtasks.forEach(subtask => {
      if (!assignments[subtask.owner]) {
        assignments[subtask.owner] = [];
      }
      assignments[subtask.owner].push(subtask);
    });

    return assignments;
  }
}
````

---

### **ขั้นที่ 5: สร้าง API Server**

สร้างไฟล์ `service/server.ts`:

````typescript
import express, { Express, Request, Response } from 'express';
import cors from 'cors';
import { AgentOrchestrator } from './orchestrator/agent-runner';
import { ProjectManagerAgent } from './agents/project-manager';
import { Task, WorkflowState } from './types/agent';

const app: Express = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(cors());
app.use(express.json());

// Initialize Orchestrator
const orchestrator = new AgentOrchestrator();
orchestrator.registerAgent(new ProjectManagerAgent());

// Routes

// ✅ สร้างงานใหม่
app.post('/api/tasks', (req: Request, res: Response) => {
  const { title, description, priority } = req.body;

  const newTask: Task = {
    id: `task-${Date.now()}`,
    title,
    description,
    status: 'pending',
    assignedAgent: 'ProjectManager',
    priority: priority || 'medium'
  };

  res.json({
    success: true,
    task: newTask,
    message: 'Task created and queued for processing'
  });
});

// ✅ รัน Workflow
app.post('/api/workflow/run', async (req: Request, res: Response) => {
  try {
    const { tasks } = req.body;
    const result = await orchestrator.runWorkflow(tasks);

    res.json({
      success: true,
      workflow: result
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: String(error)
    });
  }
});

// ✅ ได้รับสถานะ
app.get('/api/workflow/status', (req: Request, res: Response) => {
  const status = orchestrator.getStatus();

  res.json({
    success: true,
    status
  });
});

// ✅ Health Check
app.get('/api/health', (req: Request, res: Response) => {
  res.json({
    status: 'ok',
    timestamp: new Date().toISOString()
  });
});

// Start Server
app.listen(PORT, () => {
  console.log(`🚀 Agent Orchestrator running on http://localhost:${PORT}`);
});
````

---

## 📱 การเชื่อมต่อ Frontend กับ Backend

แก้ไขไฟล์ `index.html` ส่วน Alpine.js:

````javascript
// เพิ่มใน ashvalApp() function
async function connectToBackend() {
  try {
    const response = await fetch('http://localhost:3000/api/health');
    const data = await response.json();
    console.log('✅ Backend connected:', data);
    return true;
  } catch (error) {
    console.error('❌ Backend connection failed:', error);
    return false;
  }
}

// ส่งงานไปยัง Backend
async function submitTaskToWorkflow(task) {
  try {
    const response = await fetch('http://localhost:3000/api/tasks', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(task)
    });
    
    const result = await response.json();
    this.showNotification('success', 'Task submitted to workflow');
    return result;
  } catch (error) {
    this.showNotification('error', 'Failed to submit task');
  }
}

// ตรวจสอบสถานะ Workflow
async function checkWorkflowStatus() {
  try {
    const response = await fetch('http://localhost:3000/api/workflow/status');
    const data = await response.json();
    console.log('📊 Workflow Status:', data.status);
    return data.status;
  } catch (error) {
    console.error('Error checking status:', error);
  }
}
````

---

## 🔧 ไฟล์ Configuration

สร้างไฟล์ `package.json`:

````json
{
  "name": "ashval-agent-system",
  "version": "1.0.0",
  "description": "Ashval Task Assistant with Agent-based Workflow",
  "main": "service/server.ts",
  "scripts": {
    "dev": "ts-node service/server.ts",
    "build": "tsc",
    "start": "node dist/service/server.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.2",
    "cors": "^2.8.5",
    "dotenv": "^16.0.3"
  },
  "devDependencies": {
    "@types/express": "^4.17.17",
    "@types/node": "^18.11.9",
    "typescript": "^4.9.4",
    "ts-node": "^10.9.1"
  }
}
````

---

## 📊 Workflow ตัวอย่าง

```
User Input (Frontend)
        │
        ▼
┌──────────────────────┐
│ Create Task          │
│ "Build Dashboard"    │
└──────────────────────┘
        │
        ▼
┌──────────────────────────────────────────┐
│ Project Manager Agent                    │
│ • Break into subtasks                    │
│ • Assign to: Architecture, Reviewer, etc │
└──────────────────────────────────────────┘
        │
        ├─────────────────────┬──────────────┐
        ▼                     ▼              ▼
┌──────────────┐    ┌──────────────┐  ┌──────────────┐
│ Architecture │    │ Reviewer     │  │ Doc Writer   │
│ Designer     │    │              │  │              │
└──────────────┘    └──────────────┘  └──────────────┘
        │                   │              │
        └─────────────────┬─┴──────────────┘
                          ▼
                  ┌──────────────────┐
                  │ CI/CD Deployment │
                  │ Build & Test     │
                  └──────────────────┘
                          │
                          ▼
                  ┌──────────────────┐
                  │ Telemetry Agent  │
                  │ Report Results   │
                  └──────────────────┘
                          │
                          ▼
                  Frontend Dashboard
                  (Update UI)
```

---

## ✅ Checklist การผสานรวม

- [ ] ตั้งค่าโครงสร้าง Backend
- [ ] สร้าง Agent Interface
- [ ] สร้าง Agent Orchestrator
- [ ] สร้าง 7 Agents (ตามไฟล์ instructions)
- [ ] สร้าง API Server
- [ ] เชื่อมต่อ Frontend กับ Backend
- [ ] ทดสอบ Workflow ครบวงจร
- [ ] ตั้งค่า CI/CD Pipeline
- [ ] Deploy ขึ้น Production

---

## 🎯 ขั้นตอนถัดไป

1. **ติดตั้ง Dependencies:**
   ```bash
   npm install
   ```

2. **รัน Backend:**
   ```bash
   npm run dev
   ```

3. **เปิด Frontend:**
   ```
   http://localhost:5173 (หรือ file://path/to/index.html)
   ```

4. **ทดสอบการเชื่อมต่อ:**
   - ไปที่ Console ของ Browser
   - ตรวจสอบว่า Backend เชื่อมต่อได้

---

**ต้องการให้ฉันช่วยในส่วนไหนเพิ่มเติมหรือไม่?** 🚀