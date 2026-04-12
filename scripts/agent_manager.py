#!/usr/bin/env python3
import json
import os
import sys
import argparse

def load_agents(base_dir):
    """Load agents from built-in and custom locations."""
    agents = {}
    
    # Define paths
    extension_root = os.path.dirname(base_dir) # parent of scripts/
    builtin_json = os.path.join(extension_root, 'agents', 'agents.json')
    custom_json = os.path.join(extension_root, 'custom', 'agents.json')

    # Load Built-in Agents
    if os.path.exists(builtin_json):
        try:
            with open(builtin_json, 'r') as f:
                data = json.load(f)
                for agent in data.get('agents', []):
                    agent['type'] = 'built-in'
                    # Resolve full path to the .md file
                    agent['full_path'] = os.path.join(extension_root, 'agents', agent['file'])
                    agents[agent['id']] = agent
        except Exception as e:
            print(f"Warning: Failed to load built-in agents: {e}", file=sys.stderr)

    # Load Custom Agents
    if os.path.exists(custom_json):
        try:
            with open(custom_json, 'r') as f:
                data = json.load(f)
                for agent in data.get('agents', []):
                    agent['type'] = 'custom'
                    # Resolve full path, assuming relative to custom/ directory
                    agent['full_path'] = os.path.join(extension_root, 'custom', agent['file'])
                    agents[agent['id']] = agent
        except Exception as e:
            # It's okay if custom loading fails, just warn
            print(f"Warning: Failed to load custom agents: {e}", file=sys.stderr)
    
    return agents

def cmd_list(agents):
    """List all available agents in a formatted table."""
    print(f"{ 'ID':<20} { 'Name':<25} { 'Type':<10} {'Category'}")
    print("="*80)
    
    # Sort by ID
    for agent_id in sorted(agents.keys()):
        agent = agents[agent_id]
        category = agent.get('category', 'Uncategorized')
        print(f"{agent_id:<20} {agent['name']:<25} {agent['type']:<10} {category}")

def cmd_info(agents, agent_id):
    """Show detailed info for a specific agent."""
    agent = agents.get(agent_id)
    if not agent:
        print(f"Error: Agent '{agent_id}' not found.", file=sys.stderr)
        sys.exit(1)
    
    print(f"[{agent['name']}]")
    print(f"ID:          {agent['id']}")
    print(f"Type:        {agent['type']}")
    print(f"Category:    {agent.get('category', 'N/A')}")
    print(f"Description: {agent.get('description', '')}")
    print(f"Path:        {agent['full_path']}")
    
    if 'use_cases' in agent and agent['use_cases']:
        print("\nUse Cases:")
        for uc in agent['use_cases']:
            print(f"  - {uc}")
            
    if 'personality' in agent:
        print(f"\nPersonality: {agent['personality']}")

def cmd_examples(agents, agent_id=None):
    """Show example prompts for one or more agents."""
    target_agents = []
    if agent_id:
        if agent_id in agents:
            target_agents.append(agents[agent_id])
        else:
            print(f"Error: Agent '{agent_id}' not found.", file=sys.stderr)
            sys.exit(1)
    else:
        # Pick 3 diverse agents
        ids = list(agents.keys())
        import random
        selected_ids = random.sample(ids, min(3, len(ids)))
        target_agents = [agents[sid] for sid in selected_ids]

    for agent in target_agents:
        print(f"\n--- Examples for {agent['name']} ({agent['id']}) ---")
        use_cases = agent.get('use_cases', [])
        if not use_cases:
            print("No specific use cases defined.")
            continue
        
        for i, uc in enumerate(use_cases, 1):
            print(f"{i}. Prompt Idea: {uc}")

def cmd_new(base_dir):
    """Interactive wizard to create a new agent."""
    print("AI Agent Creation Wizard")
    print("========================")
    
    agent_id = input("Agent ID (kebab-case, e.g. my-expert): ").strip().lower()
    if not agent_id:
        print("Error: ID required.")
        return

    name = input("Agent Name (e.g. My Expert): ").strip()
    category = input("Category (engineering/creative/utility/comedy): ").strip().lower()
    description = input("Short Description: ").strip()
    
    # Simple prompt template
    prompt_content = f"""---
name: {agent_id}
description: {description}
category: {category}
---

You are an expert {name}. 
Your goal is to...
"""

    extension_root = os.path.dirname(base_dir)
    custom_dir = os.path.join(extension_root, 'custom')
    os.makedirs(custom_dir, exist_ok=True)
    
    md_path = os.path.join(custom_dir, f"{agent_id}.md")
    with open(md_path, 'w') as f:
        f.write(prompt_content)
    
    # Update custom registry
    registry_path = os.path.join(custom_dir, 'agents.json')
    registry = {"agents": []}
    if os.path.exists(registry_path):
        with open(registry_path, 'r') as f:
            registry = json.load(f)
    
    registry['agents'].append({
        "id": agent_id,
        "name": name,
        "file": f"{agent_id}.md",
        "category": category,
        "description": description
    })
    
    with open(registry_path, 'w') as f:
        json.dump(registry, f, indent=2)
        
    print(f"\n✅ Agent '{name}' created and registered in 'custom/'")
    print(f"Path: {md_path}")

def cmd_path(agents, agent_id):
    """Output only the absolute path of the agent file (for scripting)."""
    agent = agents.get(agent_id)
    if not agent:
        print(f"Error: Agent '{agent_id}' not found.", file=sys.stderr)
        sys.exit(1)
    print(os.path.abspath(agent['full_path']))

def main():
    parser = argparse.ArgumentParser(description="Gemini System Agent Manager")
    subparsers = parser.add_subparsers(dest='command', required=True)

    # Subcommands
    subparsers.add_parser('list', help='List all agents')
    
    info_parser = subparsers.add_parser('info', help='Get agent details')
    info_parser.add_argument('agent_id', help='Agent ID')
    
    path_parser = subparsers.add_parser('path', help='Get agent file path')
    path_parser.add_argument('agent_id', help='Agent ID')

    subparsers.add_parser('examples', help='Show agent examples').add_argument('agent_id', nargs='?', help='Agent ID')
    subparsers.add_parser('new', help='Create a new agent')

    args = parser.parse_args()
    
    # Setup paths
    script_dir = os.path.dirname(os.path.abspath(__file__))
    agents = load_agents(script_dir)

    # Dispatch
    if args.command == 'list':
        cmd_list(agents)
    elif args.command == 'info':
        cmd_info(agents, args.agent_id)
    elif args.command == 'path':
        cmd_path(agents, args.agent_id)
    elif args.command == 'examples':
        cmd_examples(agents, args.agent_id)
    elif args.command == 'new':
        cmd_new(script_dir)

if __name__ == "__main__":
    main()
