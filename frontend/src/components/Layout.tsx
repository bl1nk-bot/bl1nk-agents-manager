import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useAgentContext } from '../contexts/AgentContext';

interface MainLayoutProps {
  children: React.ReactNode;
}

export const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  const { selectedAgent } = useAgentContext();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const location = useLocation();

  const isActive = (path: string) => location.pathname === path;

  return (
    <div className="flex h-screen bg-gray-900 text-gray-100">
      {/* Sidebar */}
      <aside className={`fixed md:relative z-30 h-full transition-transform duration-300 transform ${
        sidebarOpen ? 'translate-x-0' : '-translate-x-full'
      } md:translate-x-0 w-64 bg-gray-800 border-r border-gray-700 flex flex-col`}>
        <div className="p-4 border-b border-gray-700">
          <Link to="/" className="flex items-center space-x-2">
            <div className="bg-green-500 w-8 h-8 rounded-lg flex items-center justify-center">
              <span className="font-bold text-gray-900">B</span>
            </div>
            <span className="text-xl font-bold text-green-400">Bl1nk Agents</span>
          </Link>
        </div>

        <nav className="flex-1 p-2">
          <div className="space-y-1">
            <Link
              to="/"
              className={`flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive('/') 
                  ? 'bg-green-600/20 text-green-400' 
                  : 'text-gray-300 hover:bg-gray-700 hover:text-white'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-3" viewBox="0 0 20 20" fill="currentColor">
                <path d="M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z" />
              </svg>
              Dashboard
            </Link>
            
            <Link
              to="/agents"
              className={`flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive('/agents') 
                  ? 'bg-green-600/20 text-green-400' 
                  : 'text-gray-300 hover:bg-gray-700 hover:text-white'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-3" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M12.316 3.051a1 1 0 01.633 1.265l-4 12a1 1 0 11-1.898-.632l4-12a1 1 0 011.265-.633zM5.707 6.293a1 1 0 010 1.414L3.414 10l2.293 2.293a1 1 0 11-1.414 1.414l-3-3a1 1 0 010-1.414l3-3a1 1 0 011.414 0zm8.586 0a1 1 0 011.414 0l3 3a1 1 0 010 1.414l-3 3a1 1 0 11-1.414-1.414L16.586 10l-2.293-2.293a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
              All Agents
            </Link>
            
            <Link
              to="/sessions"
              className={`flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive('/sessions') 
                  ? 'bg-green-600/20 text-green-400' 
                  : 'text-gray-300 hover:bg-gray-700 hover:text-white'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-3" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clipRule="evenodd" />
              </svg>
              Sessions
            </Link>
            
            <Link
              to="/settings"
              className={`flex items-center px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive('/settings') 
                  ? 'bg-green-600/20 text-green-400' 
                  : 'text-gray-300 hover:bg-gray-700 hover:text-white'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-3" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 00-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clipRule="evenodd" />
              </svg>
              Settings
            </Link>
          </div>
        </nav>

        <div className="p-4 border-t border-gray-700">
          <div className="flex items-center">
            {selectedAgent ? (
              <>
                <div className="bg-green-500 w-6 h-6 rounded-full flex items-center justify-center text-xs">
                  {selectedAgent.name.charAt(0)}
                </div>
                <div className="ml-2 overflow-hidden">
                  <div className="text-sm font-medium text-green-400 truncate">{selectedAgent.name}</div>
                  <div className="text-xs text-gray-400 truncate">{selectedAgent.category}</div>
                </div>
              </>
            ) : (
              <div className="text-sm text-gray-400">No agent selected</div>
            )}
          </div>
        </div>
      </aside>

      {/* Overlay for mobile */}
      {sidebarOpen && (
        <div 
          className="fixed inset-0 z-20 bg-black bg-opacity-50 md:hidden"
          onClick={() => setSidebarOpen(false)}
        ></div>
      )}

      {/* Main content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="bg-gray-800 border-b border-gray-700 p-4 flex items-center justify-between">
          <div className="flex items-center">
            <button 
              className="md:hidden mr-4 text-gray-400 hover:text-white"
              onClick={() => setSidebarOpen(!sidebarOpen)}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
            <h1 className="text-xl font-semibold">
              {location.pathname === '/' && 'Agent Dashboard'}
              {location.pathname === '/agents' && 'All Agents'}
              {location.pathname === '/sessions' && 'Active Sessions'}
              {location.pathname === '/settings' && 'Settings'}
              {(location.pathname.startsWith('/agents/') || location.pathname === '/select') && 'Agent Details'}
            </h1>
          </div>
          
          <div className="flex items-center space-x-4">
            <button className="p-2 rounded-lg bg-gray-700 hover:bg-gray-600">
              <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path d="M10 2a6 6 0 00-6 6v3.586l-.707.707A1 1 0 004 14h12a1 1 0 00.707-1.707L16 11.586V8a6 6 0 00-6-6zM10 18a3 3 0 01-3-3h6a3 3 0 01-3 3z" />
              </svg>
            </button>
            <div className="w-8 h-8 rounded-full bg-gray-700 flex items-center justify-center">
              <span className="text-sm">U</span>
            </div>
          </div>
        </header>

        {/* Content area */}
        <div className="flex-1 overflow-y-auto p-4 bg-gray-900">
          {children}
        </div>
      </main>
    </div>
  );
};