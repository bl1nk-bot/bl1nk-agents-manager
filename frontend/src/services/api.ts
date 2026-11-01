import { invoke } from '@tauri-apps/api/core';

export interface Agent {
  id: string;
  name: string;
  file: string;
  category: string;
  description: string;
  useCases: string[];
  tags: string[];
  complexity: string;
  outputs: string[];
  personality: string;
}

export interface ProcessStatus {
  conversationId: string;
  pid: number | null;
  createdAt: number;
  isAlive: boolean;
  backendType: string;
}

// Agent-related API functions
export async function getAgents(): Promise<Agent[]> {
  try {
    // In a real implementation, this would call the Tauri backend
    // For now, we'll return mock data
    return [
      {
        id: 'architect',
        name: 'Architect',
        file: 'architect.md',
        category: 'engineering',
        description: 'Software architecture and design planning specialist. Focuses exclusively on research, analysis, and documentation.',
        useCases: [
          'System design and architecture planning',
          'Technology decision documentation',
          'Creating architectural diagrams (Mermaid)',
          'Researching best practices and patterns',
          'Writing technical decision records (ADRs)'
        ],
        tags: ['engineering', 'high-complexity', 'architecture', 'design', 'documentation'],
        complexity: 'high',
        outputs: ['documentation', 'diagrams'],
        personality: 'Professional, analytical, thorough'
      },
      {
        id: 'code-generator',
        name: 'Code Generator',
        file: 'code-generator.md',
        category: 'engineering',
        description: 'Streamlined code generation assistant. Expert in all programming languages and frameworks.',
        useCases: [
          'Rapid code generation',
          'Implementing functions and classes',
          'Creating boilerplate code',
          'Writing algorithms',
          'Quick prototyping'
        ],
        tags: ['engineering', 'medium-complexity', 'code-generation', 'implementation', 'code'],
        complexity: 'medium',
        outputs: ['code'],
        personality: 'Direct, efficient, code-focused'
      },
      {
        id: 'creative-writer',
        name: 'Creative Writer',
        file: 'creative-writer.md',
        category: 'creative',
        description: 'Creative writing specialist for poetry, prose, and storytelling.',
        useCases: [
          'Writing poetry (haiku, sonnets, free verse)',
          'Creating short stories and prose',
          'Crafting dialogue and character voices',
          'Developing narratives and plots',
          'Style adaptation and literary mimicry'
        ],
        tags: ['creative', 'low-complexity', 'writing', 'persona', 'text'],
        complexity: 'low',
        outputs: ['text', 'persona'],
        personality: 'Artistic, expressive, imaginative'
      },
      {
        id: 'comedian',
        name: 'Comedian',
        file: 'comedian.md',
        category: 'entertainment',
        description: 'Dad joke specialist. Responds to any input with related dad jokes.',
        useCases: [
          'Breaking up serious work sessions',
          'Ice breakers and light conversation',
          'Testing the system with humor',
          'Entertainment and mood lifting'
        ],
        tags: ['creative', 'low-complexity', 'entertainment', 'persona', 'text'],
        complexity: 'low',
        outputs: ['persona', 'text'],
        personality: 'Punny, wholesome, cheerfully groan-worthy'
      }
    ];
  } catch (error) {
    console.error('Error fetching agents:', error);
    // Return mock data in case of error
    return [
      {
        id: 'architect',
        name: 'Architect',
        file: 'architect.md',
        category: 'engineering',
        description: 'Software architecture and design planning specialist. Focuses exclusively on research, analysis, and documentation.',
        useCases: [
          'System design and architecture planning',
          'Technology decision documentation',
          'Creating architectural diagrams (Mermaid)',
          'Researching best practices and patterns',
          'Writing technical decision records (ADRs)'
        ],
        tags: ['engineering', 'high-complexity', 'architecture', 'design', 'documentation'],
        complexity: 'high',
        outputs: ['documentation', 'diagrams'],
        personality: 'Professional, analytical, thorough'
      },
      {
        id: 'code-generator',
        name: 'Code Generator',
        file: 'code-generator.md',
        category: 'engineering',
        description: 'Streamlined code generation assistant. Expert in all programming languages and frameworks.',
        useCases: [
          'Rapid code generation',
          'Implementing functions and classes',
          'Creating boilerplate code',
          'Writing algorithms',
          'Quick prototyping'
        ],
        tags: ['engineering', 'medium-complexity', 'code-generation', 'implementation', 'code'],
        complexity: 'medium',
        outputs: ['code'],
        personality: 'Direct, efficient, code-focused'
      }
    ];
  }
}

export async function getAgentById(id: string): Promise<Agent | null> {
  try {
    const agents = await getAgents();
    return agents.find(agent => agent.id === id) || null;
  } catch (error) {
    console.error(`Error fetching agent with id ${id}:`, error);
    return null;
  }
}

// Session-related API functions
export async function startSession(
  sessionId: string, 
  workingDirectory: string, 
  model: string
): Promise<void> {
  try {
    await invoke('start_session', {
      sessionId,
      workingDirectory,
      model,
      cliName: null,
      backendConfig: null,
      geminiAuth: null,
      llxprtConfig: null
    });
  } catch (error) {
    console.error('Error starting session:', error);
    throw error;
  }
}

export async function sendMessage(
  sessionId: string,
  message: string,
  conversationHistory: string
): Promise<void> {
  try {
    await invoke('send_message', {
      sessionId,
      message,
      conversationHistory
    });
  } catch (error) {
    console.error('Error sending message:', error);
    throw error;
  }
}

export async function getProcessStatuses(): Promise<ProcessStatus[]> {
  try {
    return await invoke('get_process_statuses');
  } catch (error) {
    console.error('Error fetching process statuses:', error);
    return [];
  }
}

export async function killProcess(conversationId: string): Promise<void> {
  try {
    await invoke('kill_process', { conversationId });
  } catch (error) {
    console.error('Error killing process:', error);
    throw error;
  }
}

// Utility functions
export async function checkCliInstalled(): Promise<boolean> {
  try {
    return await invoke('check_cli_installed');
  } catch (error) {
    console.error('Error checking CLI installation:', error);
    return false;
  }
}