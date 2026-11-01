import React, { createContext, useContext, useState, useEffect } from 'react';
import { Agent } from '../types/agent';
import { getAgents } from '../services/api';

interface AgentContextType {
  agents: Agent[];
  selectedAgent: Agent | null;
  setSelectedAgent: (agent: Agent | null) => void;
  loading: boolean;
  refreshAgents: () => Promise<void>;
}

const AgentContext = createContext<AgentContextType | undefined>(undefined);

export const AgentProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);

  const refreshAgents = async () => {
    setLoading(true);
    try {
      const fetchedAgents = await getAgents();
      setAgents(fetchedAgents);
    } catch (error) {
      console.error('Error refreshing agents:', error);
      // Keep existing agents if refresh fails
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refreshAgents();
  }, []);

  return (
    <AgentContext.Provider
      value={{
        agents,
        selectedAgent,
        setSelectedAgent,
        loading,
        refreshAgents,
      }}
    >
      {children}
    </AgentContext.Provider>
  );
};

export const useAgentContext = (): AgentContextType => {
  const context = useContext(AgentContext);
  if (context === undefined) {
    throw new Error('useAgentContext must be used within an AgentProvider');
  }
  return context;
};