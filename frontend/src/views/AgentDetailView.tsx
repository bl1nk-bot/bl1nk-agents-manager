import React from 'react';
import { useParams } from 'react-router-dom';
import { useAgentContext } from '../contexts/AgentContext';

export const AgentDetailView: React.FC = () => {
  const { agentId } = useParams<{ agentId: string }>();
  const { agents, loading } = useAgentContext();

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-green-500"></div>
      </div>
    );
  }

  const agent = agents.find(a => a.id === agentId);

  if (!agent) {
    return (
      <div className="text-center py-12">
        <h2 className="text-2xl font-bold text-red-400">Agent Not Found</h2>
        <p className="text-gray-400 mt-2">The requested agent could not be found.</p>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="mb-6">
        <button className="flex items-center text-green-400 hover:text-green-300 mb-4">
          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-1" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clipRule="evenodd" />
          </svg>
          Back to Agents
        </button>
        
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <div className="flex justify-between items-start">
            <div>
              <h1 className="text-3xl font-bold text-green-400">{agent.name}</h1>
              <div className="flex items-center mt-2 space-x-4">
                <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-700 text-green-400">
                  {agent.category}
                </span>
                <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-gray-700 text-yellow-400">
                  {agent.complexity} complexity
                </span>
              </div>
            </div>
            <button className="px-4 py-2 bg-green-600 hover:bg-green-500 rounded">
              Use This Agent
            </button>
          </div>
          
          <p className="text-gray-300 mt-4">{agent.description}</p>
          
          <div className="mt-6 grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-gray-750 border border-gray-600 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-green-400 mb-3">Use Cases</h3>
              <ul className="space-y-2">
                {agent.useCases.map((useCase, index) => (
                  <li key={index} className="flex items-start">
                    <span className="text-green-500 mr-2">•</span>
                    <span className="text-gray-300">{useCase}</span>
                  </li>
                ))}
              </ul>
            </div>
            
            <div className="bg-gray-750 border border-gray-600 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-green-400 mb-3">Personality</h3>
              <p className="text-gray-300">{agent.personality}</p>
            </div>
            
            <div className="bg-gray-750 border border-gray-600 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-green-400 mb-3">Tags</h3>
              <div className="flex flex-wrap gap-2">
                {agent.tags.map((tag, index) => (
                  <span 
                    key={index}
                    className="text-xs bg-gray-700 text-green-400 px-2 py-1 rounded"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
            
            <div className="bg-gray-750 border border-gray-600 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-green-400 mb-3">Outputs</h3>
              <div className="flex flex-wrap gap-2">
                {agent.outputs.map((output, index) => (
                  <span 
                    key={index}
                    className="text-xs bg-gray-700 text-blue-400 px-2 py-1 rounded"
                  >
                    {output}
                  </span>
                ))}
              </div>
            </div>
          </div>
          
          <div className="mt-6 bg-gray-750 border border-gray-600 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-green-400 mb-3">Agent File</h3>
            <div className="bg-gray-900 p-3 rounded text-sm font-mono overflow-x-auto">
              {agent.file}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};