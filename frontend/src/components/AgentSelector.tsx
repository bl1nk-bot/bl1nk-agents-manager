import React, { useState } from 'react';
import { useAgentContext } from '../contexts/AgentContext';
import { Agent } from '../types/agent';

interface AgentSelectorProps {
  onSelect: (agent: Agent) => void;
  onCancel: () => void;
}

export const AgentSelector: React.FC<AgentSelectorProps> = ({ onSelect, onCancel }) => {
  const { agents, loading } = useAgentContext();
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  const filteredAgents = agents.filter(agent =>
    agent.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    agent.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
    agent.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()))
  );

  const handleSelect = () => {
    if (selectedAgentId) {
      const agent = agents.find(a => a.id === selectedAgentId);
      if (agent) {
        onSelect(agent);
      }
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-green-500"></div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 w-11/12 max-w-2xl max-h-3/4 overflow-hidden">
        <h2 className="text-2xl font-bold text-green-400 mb-4">Select an Agent</h2>

        <div className="mb-4">
          <input
            type="text"
            placeholder="Search agents..."
            className="w-full bg-gray-700 text-white rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-green-500"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div className="space-y-3 max-h-64 overflow-y-auto pr-2 mb-4">
          {filteredAgents.map((agent) => (
            <div
              key={agent.id}
              className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                selectedAgentId === agent.id
                  ? 'border-green-500 bg-gray-700'
                  : 'border-gray-600 hover:border-gray-500 hover:bg-gray-750'
              }`}
              onClick={() => setSelectedAgentId(agent.id)}
            >
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="font-medium text-green-400">{agent.name}</h3>
                  <p className="text-sm text-gray-400 mt-1">{agent.description}</p>
                </div>
                <span className="text-xs bg-gray-700 text-gray-300 px-2 py-1 rounded-full">
                  {agent.category}
                </span>
              </div>
              <div className="mt-2 flex flex-wrap gap-1">
                {agent.tags.slice(0, 3).map((tag, index) => (
                  <span
                    key={index}
                    className="text-xs bg-gray-700 text-green-400 px-2 py-1 rounded"
                  >
                    {tag}
                  </span>
                ))}
                {agent.tags.length > 3 && (
                  <span className="text-xs bg-gray-700 text-gray-400 px-2 py-1 rounded">
                    +{agent.tags.length - 3} more
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>

        {filteredAgents.length === 0 && (
          <div className="text-center py-8 text-gray-400">
            No agents found matching your search.
          </div>
        )}

        <div className="flex justify-end space-x-3">
          <button
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            className={`px-4 py-2 rounded-lg ${
              selectedAgentId
                ? 'bg-green-600 hover:bg-green-500'
                : 'bg-gray-700 cursor-not-allowed'
            }`}
            onClick={handleSelect}
            disabled={!selectedAgentId}
          >
            Select Agent
          </button>
        </div>
      </div>
    </div>
  );
};