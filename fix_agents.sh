#!/bin/bash

# Script to fix agent files that are missing the frontmatter separator

cd /home/bill/dev/tools/extension/agents-manager/agents

# List of files that need to be fixed based on the validation error
files=(
    "plugin-validator.md"
    "reporter.md"
    "pr-reviewer.md"
    "docs-researcher.md"
    "deep-reflector.md"
    "thoughts-locator.md"
    "ui-engineer.md"
    "instruction-reflector.md"
    "insight-documenter.md"
    "testing-guidelines.md"
    "plan-implementation-reviewer.md"
    "code-architect.md"
    "orchestrator.md"
    "prompt-engineering-patterns.md"
    "workflow-diagrams.md"
    "prompt-template-system.md"
    "fullstack-dev.md"
    "codebase-pattern-finder.md"
    "research-analyzer.md"
    "pdf-to-ppt.md"
    "code-generator.md"
    "codebase-analyzer.md"
    "web-search-researcher.md"
    "codebase-locator.md"
    "task-management.md"
    "hookify-conversation-analyzer.md"
    "command-creator.md"
    "code-reviewer.md"
    "kiro-workflow.md"
    "thoughts-analyzer.md"
    "skill-creator.md"
    "github-issue-fixer.md"
    "docbot-pro.md"
    "skill-reviewer.md"
    "code-explorer.md"
    "cloudflare.md"
    "human-3-coach.md"
    "creative-writer.md"
    "README_NEW.md"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        # Check if the file already starts with ---
        first_line=$(head -n 1 "$file")
        if [ "$first_line" != "---" ]; then
            # Extract the name from the first line (before the colon)
            name=$(echo "$first_line" | cut -d':' -f1 | cut -d' ' -f1)
            
            # If name is not properly extracted, use the filename without extension
            if [ -z "$name" ] || [ "$name" = "$file" ]; then
                name="${file%.md}"
            fi
            
            # Create a temporary file with the correct frontmatter
            temp_file=$(mktemp)
            
            # Add the proper frontmatter
            echo "---" > "$temp_file"
            echo "name: $name" >> "$temp_file"
            echo "description: $(echo "$first_line" | cut -d':' -f2- | sed 's/^ *//')" >> "$temp_file"
            echo "category: utility" >> "$temp_file"
            echo "model: optional_model_name" >> "$temp_file"
            echo "color: \"#9B59B6\"" >> "$temp_file"
            echo "---" >> "$temp_file"
            
            # Add the rest of the original file content
            tail -n +2 "$file" >> "$temp_file"
            
            # Replace the original file with the fixed version
            mv "$temp_file" "$file"
            
            echo "Fixed: $file"
        else
            echo "Already correct: $file"
        fi
    else
        echo "File not found: $file"
    fi
done

echo "All agent files have been fixed."