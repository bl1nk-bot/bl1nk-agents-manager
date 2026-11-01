import React from 'react';
import { Agent } from '../types/agent';
import { Link } from 'react-router-dom';

interface AgentCardProps {
  agent: Agent;
}

export const AgentCard: React.FC<AgentCardProps> = ({ agent }) => {
  return (
    <Link to={`/agents/${agent.id}`} className="block">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 hover:border-green-500 transition-colors">
        <div className="flex justify-between items-start">
          <h3 className="text-lg font-semibold text-green-400">{agent.name}</h3>
          <span className="text-xs bg-gray-700 text-green-400 px-2 py-1 rounded-full">
            {agent.complexity}
          </span>
        </div>
        
        <p className="text-gray-300 mt-2 text-sm">{agent.description}</p>
        
        <div className="mt-4 flex flex-wrap gap-2">
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
        
        <div className="mt-4 pt-3 border-t border-gray-700 flex justify-between items-center">
          <span className="text-xs text-gray-400">{agent.category}</span>
          <button className="text-xs bg-green-600 hover:bg-green-500 text-white px-3 py-1 rounded">
            Use Agent
          </button>
        </div>
      </div>
    </Link>
  );
};