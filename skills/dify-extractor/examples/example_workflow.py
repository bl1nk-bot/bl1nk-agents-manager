#!/usr/bin/env python3
"""
Example Workflow: Dify Extractor with Contact Management
Shows typical usage patterns for document extraction and contact preservation
"""

import sys
import json
from pathlib import Path

# Add parent directories to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'scripts'))
sys.path.insert(0, str(Path(__file__).parent.parent / 'src' / 'providers'))

from dify_extractor import (
    DifyExtractorContext,
    DocumentExtractor,
    ContactManager
)


def example_1_extract_csv_with_contacts():
    """Example 1: Extract CSV and preserve contacts"""
    print("\n" + "="*60)
    print("Example 1: Extract CSV with Contact Preservation")
    print("="*60)
    
    context = DifyExtractorContext()
    extractor = DocumentExtractor(context, verbose=True)
    
    # Simulate CSV extraction
    result = {
        'status': 'success',
        'data': [
            {
                'name': 'John Doe',
                'email': 'john@example.com',
                'phone': '555-1234',
                'company': 'ACME Corp'
            },
            {
                'name': 'Jane Smith',
                'email': 'jane@example.com',
                'phone': '555-5678',
                'company': 'Tech Inc'
            }
        ],
        'context': context.to_dict()
    }
    
    print(json.dumps(result, indent=2))
    return context


def example_2_manage_contacts(context):
    """Example 2: Store and retrieve contacts"""
    print("\n" + "="*60)
    print("Example 2: Contact Management")
    print("="*60)
    
    contact_manager = ContactManager(context)
    
    # Store contact
    contact_data = {
        'name': 'John Doe',
        'email': 'john@example.com',
        'phone': '555-1234',
        'company': 'ACME Corp',
        'department': 'Sales'
    }
    
    store_result = contact_manager.store_contact('john@example.com', contact_data)
    print("\nStore Contact Result:")
    print(json.dumps(store_result, indent=2))
    
    # Retrieve contact
    retrieve_result = contact_manager.retrieve_contact('john@example.com')
    print("\nRetrieve Contact Result:")
    print(json.dumps(retrieve_result, indent=2, default=str))
    
    # List all contacts
    list_result = contact_manager.list_contacts()
    print("\nList Contacts Result:")
    print(json.dumps(list_result, indent=2, default=str))


def example_3_dify_plugin_integration():
    """Example 3: Dify Plugin Integration"""
    print("\n" + "="*60)
    print("Example 3: Dify Plugin Integration")
    print("="*60)
    
    try:
        from dify_extractor_provider import DifyExtractorProvider
        
        provider = DifyExtractorProvider()
        
        # Show available tools
        schema = provider.get_tool_schema()
        print("\nAvailable Tools:")
        for tool in schema['tools']:
            print(f"  - {tool['name']}: {tool['description']}")
        
        # Simulate tool invocation
        print("\nExample Tool Invocation (get_context):")
        result = provider.invoke('get_context', {})
        print(json.dumps(result, indent=2, default=str))
    
    except ImportError:
        print("Note: dify_plugin not installed. Showing mock example only.")


def example_4_batch_processing():
    """Example 4: Batch processing workflow"""
    print("\n" + "="*60)
    print("Example 4: Batch Processing Workflow")
    print("="*60)
    
    context = DifyExtractorContext()
    
    # Simulate processing multiple files
    files = [
        {'name': 'contacts_2024.csv', 'type': 'csv', 'records': 150},
        {'name': 'sales_data.xlsx', 'type': 'excel', 'records': 320},
        {'name': 'report.pdf', 'type': 'pdf', 'pages': 25}
    ]
    
    total_records = 0
    for file_info in files:
        context.current_document = file_info['name']
        records = file_info.get('records') or file_info.get('pages', 0)
        context.extracted_records += records
        total_records += records
        
        context.add_extraction_history('batch_process', {
            'file': file_info['name'],
            'type': file_info['type'],
            'records': records
        })
    
    result = {
        'status': 'success',
        'batch_summary': {
            'files_processed': len(files),
            'total_records': total_records,
            'contacts_found': len(context.contacts_registry)
        },
        'context': context.to_dict()
    }
    
    print(json.dumps(result, indent=2, default=str))


def main():
    print("\n" + "="*60)
    print("Dify Extractor - Example Workflows")
    print("="*60)
    
    # Run examples
    context = example_1_extract_csv_with_contacts()
    example_2_manage_contacts(context)
    example_3_dify_plugin_integration()
    example_4_batch_processing()
    
    print("\n" + "="*60)
    print("Examples completed!")
    print("="*60 + "\n")


if __name__ == '__main__':
    main()
