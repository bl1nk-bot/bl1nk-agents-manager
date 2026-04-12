"""
Dify Plugin Provider for Document Extraction
Integrates with Dify plugin ecosystem for seamless document processing
"""

from typing import Any, Dict, Optional
import json
import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'scripts'))

try:
    from dify_plugin import ToolProvider
    from dify_plugin.errors.tool import ToolProviderCredentialValidationError
except ImportError:
    # Fallback for testing without dify_plugin
    class ToolProvider:
        pass
    class ToolProviderCredentialValidationError(Exception):
        pass

from dify_extractor import (
    DifyExtractorContext,
    DocumentExtractor,
    ContactManager
)


class DifyExtractorProvider(ToolProvider):
    """
    Dify Plugin Provider for Document Extraction with Contact Management
    
    Features:
    - Extract data from CSV, Excel, PDF
    - Manage contact information
    - Preserve context across operations
    - Integration with Claude skill system
    """
    
    def __init__(self):
        super().__init__()
        self.context = DifyExtractorContext()
        self.extractor = DocumentExtractor(self.context, verbose=True)
        self.contact_manager = ContactManager(self.context)
    
    def _validate_credentials(self, credentials: dict[str, Any]) -> None:
        """Validate provider credentials"""
        try:
            # Basic validation - can be extended based on requirements
            if credentials and isinstance(credentials, dict):
                pass  # Credentials are optional for local file operations
        except Exception as e:
            raise ToolProviderCredentialValidationError(str(e))
    
    def invoke(self, tool_name: str, parameters: dict[str, Any]) -> dict[str, Any]:
        """
        Invoke extraction tool with given parameters
        
        Args:
            tool_name: Tool identifier (extract_csv, extract_excel, etc.)
            parameters: Tool parameters
        
        Returns:
            Tool execution result with status and data
        """
        try:
            if tool_name == 'extract_csv':
                return self.extractor.extract_csv(
                    input_path=parameters.get('input'),
                    output_format=parameters.get('output', 'json'),
                    preserve_contacts=parameters.get('preserve_contacts', True),
                    filter_by=parameters.get('filter_by'),
                    limit=parameters.get('limit')
                )
            
            elif tool_name == 'extract_excel':
                return self.extractor.extract_excel(
                    input_path=parameters.get('input'),
                    sheet_name=parameters.get('sheet_name'),
                    output_format=parameters.get('output', 'json'),
                    preserve_contacts=parameters.get('preserve_contacts', True),
                    header_row=parameters.get('header_row', 0)
                )
            
            elif tool_name == 'extract_pdf':
                return self.extractor.extract_pdf(
                    input_path=parameters.get('input'),
                    mode=parameters.get('mode', 'text'),
                    preserve_contacts=parameters.get('preserve_contacts', True),
                    page_range=parameters.get('page_range')
                )
            
            elif tool_name == 'store_contact':
                contact_data = parameters.get('contact_data')
                if isinstance(contact_data, str):
                    contact_data = json.loads(contact_data)
                
                return self.contact_manager.store_contact(
                    contact_id=parameters.get('contact_id'),
                    contact_data=contact_data
                )
            
            elif tool_name == 'retrieve_contact':
                return self.contact_manager.retrieve_contact(
                    contact_id=parameters.get('contact_id')
                )
            
            elif tool_name == 'list_contacts':
                return self.contact_manager.list_contacts(
                    search_query=parameters.get('search_query')
                )
            
            elif tool_name == 'get_context':
                return {
                    'status': 'success',
                    'context': self.context.to_dict()
                }
            
            else:
                return {
                    'status': 'error',
                    'error': f'Unknown tool: {tool_name}'
                }
        
        except Exception as e:
            return {
                'status': 'error',
                'error': str(e),
                'context': self.context.to_dict()
            }
    
    def get_tool_schema(self) -> dict[str, Any]:
        """
        Get schema for all available tools
        
        Returns:
            Dictionary containing tool schemas
        """
        return {
            'tools': [
                {
                    'name': 'extract_csv',
                    'description': 'Extract structured data from CSV files',
                    'parameters': {
                        'type': 'object',
                        'properties': {
                            'input': {'type': 'string', 'description': 'CSV file path'},
                            'output': {
                                'type': 'string',
                                'enum': ['json', 'dict', 'markdown'],
                                'description': 'Output format'
                            },
                            'preserve_contacts': {
                                'type': 'boolean',
                                'description': 'Extract and preserve contact information'
                            },
                            'filter_by': {'type': 'string', 'description': 'Filter column name'},
                            'limit': {'type': 'integer', 'description': 'Maximum rows to extract'}
                        },
                        'required': ['input']
                    }
                },
                {
                    'name': 'extract_excel',
                    'description': 'Extract structured data from Excel files',
                    'parameters': {
                        'type': 'object',
                        'properties': {
                            'input': {'type': 'string', 'description': 'Excel file path'},
                            'sheet_name': {'type': 'string', 'description': 'Sheet name'},
                            'output': {
                                'type': 'string',
                                'enum': ['json', 'dict', 'markdown'],
                                'description': 'Output format'
                            },
                            'preserve_contacts': {
                                'type': 'boolean',
                                'description': 'Extract and preserve contact information'
                            },
                            'header_row': {'type': 'integer', 'description': 'Header row index'}
                        },
                        'required': ['input']
                    }
                },
                {
                    'name': 'extract_pdf',
                    'description': 'Extract text and structured data from PDF files',
                    'parameters': {
                        'type': 'object',
                        'properties': {
                            'input': {'type': 'string', 'description': 'PDF file path'},
                            'mode': {
                                'type': 'string',
                                'enum': ['text', 'table', 'both'],
                                'description': 'Extraction mode'
                            },
                            'preserve_contacts': {
                                'type': 'boolean',
                                'description': 'Extract and preserve contact information'
                            },
                            'page_range': {'type': 'string', 'description': 'Pages to extract'}
                        },
                        'required': ['input']
                    }
                },
                {
                    'name': 'store_contact',
                    'description': 'Store contact information',
                    'parameters': {
                        'type': 'object',
                        'properties': {
                            'contact_id': {'type': 'string', 'description': 'Contact identifier'},
                            'contact_data': {'type': 'object', 'description': 'Contact information'}
                        },
                        'required': ['contact_id', 'contact_data']
                    }
                },
                {
                    'name': 'retrieve_contact',
                    'description': 'Retrieve stored contact information',
                    'parameters': {
                        'type': 'object',
                        'properties': {
                            'contact_id': {'type': 'string', 'description': 'Contact identifier'}
                        },
                        'required': ['contact_id']
                    }
                },
                {
                    'name': 'list_contacts',
                    'description': 'List all stored contacts with optional search',
                    'parameters': {
                        'type': 'object',
                        'properties': {
                            'search_query': {'type': 'string', 'description': 'Search query'}
                        }
                    }
                },
                {
                    'name': 'get_context',
                    'description': 'Get current extraction context and state',
                    'parameters': {
                        'type': 'object',
                        'properties': {}
                    }
                }
            ]
        }
