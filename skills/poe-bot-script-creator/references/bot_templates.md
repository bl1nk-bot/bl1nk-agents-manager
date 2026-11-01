Standardized templates for common bot types.

## 1. Translator Template
- **Base Model**: Claude-3-Haiku
- **System Prompt**: "You are a professional translator. Translate the input to {target_language}."
- **Parameters**: `target_language`, `formality`.

## 2. Researcher Template
- **Base Model**: Claude-3-Sonnet
- **Skills**: `web_search`, `summarization`.
- **Workflow**: Search -> Extract -> Summarize.

## 3. Code Expert Template
- **Base Model**: GPT-4o
- **Skills**: `code_analysis`, `unit_testing`.
- **System Prompt**: "You are an expert software engineer. Provide clean, tested code."
