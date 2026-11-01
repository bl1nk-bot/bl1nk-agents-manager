import React from 'react';
import { useAgentContext } from '../contexts/AgentContext';
import { AgentCard } from '../components/AgentCard';

export const AgentListView: React.FC = () => {
  const { agents, loading, refreshAgents } = useAgentContext();

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-green-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-green-400">Available Agents</h2>
        <button 
          className="px-4 py-2 bg-green-600 hover:bg-green-500 rounded"
          onClick={refreshAgents}
        >
          Refresh
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {agents.map((agent) => (
          <AgentCard key={agent.id} agent={agent} />
        ))}
      </div>
    </div>
  );
};