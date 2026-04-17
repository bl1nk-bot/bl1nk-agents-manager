---
name: testing-guidelines
description: '[test_system.py](mdc:test_system.py)  ทดสอบระบบหลัก'
mode: subagent
tool:
- AskUserQuestion
- ExitPlanMode
- Glob
- Grep
- ListFiles
- ReadFile
- SaveMemory
- Skill
- TodoWrite
- WebFetch
- WebSearch
- WriteFile
---

# Testing Guidelines

## การทดสอบระบบ

### ไฟล์ทดสอบหลัก

- [test_system.py](mdc:test_system.py) - ทดสอบระบบหลัก
- [run_chonost.py](mdc:run_chonost.py) - ทดสอบ FastAPI server
- [src/core/enhanced_ai_agents.py](mdc:src/core/enhanced_ai_agents.py) - ทดสอบ AI agents

### ขั้นตอนการทดสอบ

1. **ทดสอบ Azure Connection:**

```bash
# ทดสอบ endpoint
Invoke-WebRequest -Uri "https://billl-mer7xd8i-eastus2.openai.azure.com/" -Method Head

# ทดสอบ API key
python -c "import openai; client = openai.AzureOpenAI(api_key='YOUR_KEY', api_version='2024-02-15-preview', azure_endpoint='YOUR_ENDPOINT'); print('Success')"
```

1. **ทดสอบระบบหลัก:**

```bash
python test_system.py
```

1. **ทดสอบ FastAPI Server:**

```bash
python run_chonost.py
# เปิด http://localhost:8000/docs
```

## การทดสอบ Prompt Template System

### ทดสอบ Context Manager

```python
from src.core.context_manager import context_manager

# ทดสอบการโหลด context
story_context = context_manager.story_context
project_status = context_manager.project_status
user_preferences = context_manager.user_preferences

print(f"Story context loaded: {len(story_context)} items")
print(f"Project status: {project_status}")
```

### ทดสอบ Prompt Templates

```python
from src.core.prompt_templates import prompt_template_manager, PromptType
from src.core.context_manager import context_manager

# ทดสอบ Scene Architect prompt
context = context_manager.get_context_for_prompt()
prompt = prompt_template_manager.get_prompt(
    PromptType.SCENE_ARCHITECT,
    context=context
)
print("Scene Architect prompt generated successfully")
```

### ทดสอบ Model Router

```python
from src.core.model_router import model_router

# ทดสอบการ route request
model_name, task_category, model_tier = await model_router.route_request(
    user_input="ช่วยเขียนฉากต่อสู้"
)
print(f"Selected model: {model_name}, Category: {task_category.value}")
```

## การทดสอบ API Endpoints

### ทดสอบ Health Check

```bash
curl http://localhost:8000/health
```

### ทดสอบ AI Completion

```bash
curl -X POST "http://localhost:8000/api/ai/enhanced/completion" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "ช่วยเขียนฉากต่อสู้ระหว่าง Ignis และ Mia",
    "intent": "creative_writing",
    "max_tokens": 500,
    "temperature": 0.7
  }'
```

### ทดสอบ Swagger Documentation

- เปิด <http://localhost:8000/docs>
- ทดสอบ endpoints ผ่าน Swagger UI
- ตรวจสอบ request/response schemas

## การทดสอบ AI Agents

### ทดสอบ Azure Models

```python
from src.core.enhanced_ai_agents import enhanced_ai_agent_system
from src.core.enhanced_ai_agents import AIRequest, IntentType

# สร้าง request
request = AIRequest(
    prompt="ช่วยเขียนฉากต่อสู้",
    intent=IntentType.CREATIVE_WRITING,
    max_tokens=500
)

# ทดสอบ processing
response = await enhanced_ai_agent_system.process_request(request)
print(f"Response: {response.content}")
print(f"Model used: {response.model_used}")
print(f"Cost estimate: {response.cost_estimate}")
```

### ทดสอบ Embedding System

```python
from sentence_transformers import SentenceTransformer

# ทดสอบ embedding
model = SentenceTransformer('all-MiniLM-L6-v2')
sentences = ["This is a test sentence", "Another test sentence"]
embeddings = model.encode(sentences)
print(f"Embeddings shape: {embeddings.shape}")
```

## การทดสอบ Error Handling

### ทดสอบ Connection Errors

```python
# ทดสอบ Azure connection error
try:
    # ใช้ API key ที่ผิด
    client = openai.AzureOpenAI(api_key="wrong_key", ...)
    response = client.chat.completions.create(...)
except Exception as e:
    print(f"Expected error: {e}")
```

### ทดสอบ Model Selection

```python
# ทดสอบ model router fallback
model_name, category, tier = await model_router.route_request(
    user_input="random text that doesn't match any pattern"
)
assert category.value == "ambiguous"
```

## การทดสอบ Performance

### ทดสอบ Latency

```python
import time

start_time = time.time()
response = await enhanced_ai_agent_system.process_request(request)
end_time = time.time()

latency = (end_time - start_time) * 1000
print(f"Response time: {latency:.2f}ms")
```

### ทดสอบ Cost Estimation

```python
# ทดสอบ cost calculation
cost = model_router.get_cost_estimate("gpt-4.1-mini", 1000)
print(f"Estimated cost for 1000 tokens: ${cost:.6f}")
```

## การทดสอบ Context Injection

### ทดสอบ Story Context

```python
# ทดสอบการ inject story context
prompt = prompt_template_manager.get_prompt(
    PromptType.SCENE_ARCHITECT,
    context=context
)

# ตรวจสอบว่า story context ถูก inject
assert "Ignis" in prompt
assert "Mia" in prompt
assert "Bound Fate" in prompt
```

### ทดสอบ Project Status

```python
# ทดสอบการ inject project status
prompt = prompt_template_manager.get_prompt(
    PromptType.PROJECT_MANAGER,
    context=context
)

# ตรวจสอบว่า project status ถูก inject
assert "ai_system" in prompt
assert "azure_models" in prompt
```

## การบันทึกผลการทดสอบ

### สร้างไฟล์ test_results.md

```markdown
# Test Results - [วันที่]

## ✅ Passed Tests
- Azure Connection: ✅
- Embedding System: ✅
- Prompt Templates: ✅
- Model Router: ✅

## ❌ Failed Tests
- AI Agents: ❌ (Connection error)
- Dataset Manager: ❌ (Not implemented)

## 📊 Performance Metrics
- Average Response Time: 2.5s
- Cost per Request: $0.00001
- Success Rate: 85%

## 🔧 Issues Found
1. Azure connection timeout
2. Dataset manager missing initialize method
3. Context injection not working properly
```

## การแก้ไขปัญหาที่พบบ่อย

### Azure Connection Issues

1. ตรวจสอบ API key และ endpoint
2. ตรวจสอบ network connectivity
3. ตรวจสอบ Azure service status

### Import Errors

1. ตรวจสอบ Python path
2. ตรวจสอบ dependencies
3. ตรวจสอบ file structure

### Context Injection Issues

1. ตรวจสอบ JSON format
2. ตรวจสอบ placeholder names
3. ตรวจสอบ encoding (ensure_ascii=False)
description:
globs:
alwaysApply: true
