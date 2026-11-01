import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { AgentProvider } from './contexts/AgentContext';
import { MainLayout } from './components/Layout';
import { AgentDashboard } from './views/AgentDashboard';
import { AgentDetailView } from './views/AgentDetailView';
import { AgentSelector } from './components/AgentSelector';
import './App.css';

function App() {
  return (
    <AgentProvider>
      <MainLayout>
        <Routes>
          <Route path="/" element={<AgentDashboard />} />
          <Route path="/agents" element={<AgentDashboard />} />
          <Route path="/agents/:agentId" element={<AgentDetailView />} />
          <Route path="/select" element={<AgentSelector />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </MainLayout>
    </AgentProvider>
  );
}

export default App;