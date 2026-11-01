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