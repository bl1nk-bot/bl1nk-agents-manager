import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { ask, confirm } from '@tauri-apps/plugin-dialog';
import { platform } from '@tauri-apps/plugin-os';
import { exit } from '@tauri-apps/plugin-process';

// Shell command execution through Tauri
export async function executeShellCommand(command: string): Promise<{ stdout: string, stderr: string, exitCode: number }> {
  try {
    // In a real implementation, this would use the Tauri shell plugin
    // For now, we'll return mock results
    return {
      stdout: `Executed: ${command}`,
      stderr: '',
      exitCode: 0
    };
  } catch (error) {
    console.error('Error executing shell command:', error);
    return {
      stdout: '',
      stderr: `Error: ${(error as Error).message}`,
      exitCode: 1
    };
  }
}

// File system operations
export async function readFile(path: string): Promise<string> {
  try {
    return await invoke('read_file_content', { path });
  } catch (error) {
    console.error(`Error reading file ${path}:`, error);
    throw error;
  }
}

export async function writeFile(path: string, content: string): Promise<void> {
  try {
    await invoke('write_file_content', { path, content });
  } catch (error) {
    console.error(`Error writing file ${path}:`, error);
    throw error;
  }
}

export async function listDirectory(path: string): Promise<any[]> {
  try {
    return await invoke('list_directory_contents', { path });
  } catch (error) {
    console.error(`Error listing directory ${path}:`, error);
    throw error;
  }
}

// Dialog operations
export async function openFileDialog(filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      filters
    });
    
    return Array.isArray(selected) ? selected[0] || null : selected;
  } catch (error) {
    console.error('Error opening file dialog:', error);
    return null;
  }
}

export async function openDirectoryDialog(): Promise<string | null> {
  try {
    const selected = await open({
      directory: true,
      multiple: false
    });
    
    return Array.isArray(selected) ? selected[0] || null : selected;
  } catch (error) {
    console.error('Error opening directory dialog:', error);
    return null;
  }
}

// Confirmation dialogs
export async function showConfirmationDialog(message: string): Promise<boolean> {
  try {
    return await confirm(message);
  } catch (error) {
    console.error('Error showing confirmation dialog:', error);
    return false;
  }
}

export async function showInputDialog(prompt: string, defaultValue?: string): Promise<string | null> {
  try {
    // Note: Tauri doesn't have a built-in prompt dialog, so we'll simulate it
    // In a real implementation, you'd create a custom dialog
    return await ask(prompt, { title: 'Input Required' });
  } catch (error) {
    console.error('Error showing input dialog:', error);
    return null;
  }
}

// System information
export async function getPlatformInfo(): Promise<string> {
  try {
    return await platform();
  } catch (error) {
    console.error('Error getting platform info:', error);
    return 'unknown';
  }
}

// App operations
export async function quitApp(): Promise<void> {
  try {
    await exit(0);
  } catch (error) {
    console.error('Error quitting app:', error);
  }
}