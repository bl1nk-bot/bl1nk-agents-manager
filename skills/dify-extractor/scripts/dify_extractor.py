#!/usr/bin/env python3
"""
Dify Extractor Main Script
Handles document extraction (CSV, Excel, PDF) with contact management and context preservation
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional
import logging
from datetime import datetime

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class DifyExtractorContext:
    """Manages extraction context and document state"""
    
    def __init__(self):
        self.current_document = None
        self.extracted_records = 0
        self.contacts_registry = {}
        self.extraction_history = []
        self.metadata = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Export context as dictionary"""
        return {
            'current_document': self.current_document,
            'extracted_records': self.extracted_records,
            'contacts_found': len(self.contacts_registry),
            'last_operation': self.extraction_history[-1] if self.extraction_history else None,
            'timestamp': datetime.now().isoformat(),
            'metadata': self.metadata
        }
    
    def add_extraction_history(self, operation: str, details: Dict[str, Any]):
        """Record extraction operation"""
        self.extraction_history.append({
            'operation': operation,
            'timestamp': datetime.now().isoformat(),
            'details': details
        })

class DocumentExtractor:
    """Main document extraction handler"""
    
    def __init__(self, context: DifyExtractorContext, verbose: bool = False):
        self.context = context
        self.verbose = verbose
    
    def extract_csv(self, input_path: str, output_format: str = 'json', 
                   preserve_contacts: bool = True, filter_by: Optional[str] = None,
                   limit: Optional[int] = None) -> Dict[str, Any]:
        """Extract data from CSV file"""
        try:
            import pandas as pd
            
            file_path = Path(input_path)
            if not file_path.exists():
                raise FileNotFoundError(f"CSV file not found: {input_path}")
            
            df = pd.read_csv(file_path)
            
            if limit:
                df = df.head(limit)
            
            if filter_by:
                # Simple filter implementation
                pass
            
            self.context.current_document = str(file_path)
            self.context.extracted_records = len(df)
            
            if preserve_contacts:
                self._extract_and_preserve_contacts(df)
            
            result = self._format_output(df, output_format)
            
            self.context.add_extraction_history('extract_csv', {
                'file': input_path,
                'rows': len(df),
                'columns': list(df.columns),
                'contacts_found': len(self.context.contacts_registry)
            })
            
            if self.verbose:
                logger.info(f"✓ Extracted {len(df)} rows from {input_path}")
            
            return {
                'status': 'success',
                'data': result,
                'context': self.context.to_dict()
            }
        
        except Exception as e:
            logger.error(f"Error extracting CSV: {str(e)}")
            return {
                'status': 'error',
                'error': str(e),
                'context': self.context.to_dict()
            }
    
    def extract_excel(self, input_path: str, sheet_name: Optional[str] = None,
                     output_format: str = 'json', preserve_contacts: bool = True,
                     header_row: int = 0) -> Dict[str, Any]:
        """Extract data from Excel file"""
        try:
            import pandas as pd
            
            file_path = Path(input_path)
            if not file_path.exists():
                raise FileNotFoundError(f"Excel file not found: {input_path}")
            
            if sheet_name:
                df = pd.read_excel(file_path, sheet_name=sheet_name, header=header_row)
            else:
                df = pd.read_excel(file_path, header=header_row)
            
            self.context.current_document = str(file_path)
            self.context.extracted_records = len(df)
            
            if preserve_contacts:
                self._extract_and_preserve_contacts(df)
            
            result = self._format_output(df, output_format)
            
            self.context.add_extraction_history('extract_excel', {
                'file': input_path,
                'sheet': sheet_name or 'default',
                'rows': len(df),
                'columns': list(df.columns),
                'contacts_found': len(self.context.contacts_registry)
            })
            
            if self.verbose:
                logger.info(f"✓ Extracted {len(df)} rows from {input_path}")
            
            return {
                'status': 'success',
                'data': result,
                'context': self.context.to_dict()
            }
        
        except Exception as e:
            logger.error(f"Error extracting Excel: {str(e)}")
            return {
                'status': 'error',
                'error': str(e),
                'context': self.context.to_dict()
            }
    
    def extract_pdf(self, input_path: str, mode: str = 'text',
                   preserve_contacts: bool = True, page_range: Optional[str] = None) -> Dict[str, Any]:
        """Extract text and structured data from PDF"""
        try:
            import pypdf
            
            file_path = Path(input_path)
            if not file_path.exists():
                raise FileNotFoundError(f"PDF file not found: {input_path}")
            
            extracted_text = []
            with open(file_path, 'rb') as pdf_file:
                reader = pypdf.PdfReader(pdf_file)
                num_pages = len(reader.pages)
                
                for page_num in range(num_pages):
                    page = reader.pages[page_num]
                    text = page.extract_text()
                    extracted_text.append(text)
            
            self.context.current_document = str(file_path)
            self.context.extracted_records = len(extracted_text)
            
            if preserve_contacts:
                self._extract_contacts_from_text(extracted_text)
            
            self.context.add_extraction_history('extract_pdf', {
                'file': input_path,
                'pages': num_pages,
                'mode': mode,
                'contacts_found': len(self.context.contacts_registry)
            })
            
            if self.verbose:
                logger.info(f"✓ Extracted {len(extracted_text)} pages from {input_path}")
            
            return {
                'status': 'success',
                'data': extracted_text if mode == 'text' else {'text': extracted_text},
                'context': self.context.to_dict()
            }
        
        except Exception as e:
            logger.error(f"Error extracting PDF: {str(e)}")
            return {
                'status': 'error',
                'error': str(e),
                'context': self.context.to_dict()
            }
    
    def _extract_and_preserve_contacts(self, df):
        """Extract contact information from dataframe"""
        contact_columns = ['email', 'phone', 'name', 'contact', 'mail', 'phone_number']
        
        for col in df.columns:
            if any(contact_col in col.lower() for contact_col in contact_columns):
                for idx, value in enumerate(df[col]):
                    if pd.notna(value) and str(value).strip():
                        contact_id = f"contact_{idx}_{col}"
                        self.context.contacts_registry[contact_id] = {
                            'value': str(value),
                            'column': col,
                            'row_index': idx,
                            'extracted_at': datetime.now().isoformat()
                        }
    
    def _extract_contacts_from_text(self, text_list: List[str]):
        """Extract contact information from text"""
        import re
        
        email_pattern = r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
        phone_pattern = r'[\d\-\+\(\)\s]{10,}'
        
        for idx, text in enumerate(text_list):
            emails = re.findall(email_pattern, text)
            phones = re.findall(phone_pattern, text)
            
            for email in emails:
                self.context.contacts_registry[email] = {
                    'type': 'email',
                    'value': email,
                    'page': idx,
                    'extracted_at': datetime.now().isoformat()
                }
            
            for phone in phones:
                phone_id = f"phone_{idx}_{hash(phone)}"
                self.context.contacts_registry[phone_id] = {
                    'type': 'phone',
                    'value': phone,
                    'page': idx,
                    'extracted_at': datetime.now().isoformat()
                }
    
    def _format_output(self, df, output_format: str) -> str:
        """Format extraction output"""
        import pandas as pd
        
        if output_format == 'json':
            return df.to_json(orient='records', default_handler=str)
        elif output_format == 'dict':
            return df.to_dict('records')
        elif output_format == 'markdown':
            return df.to_markdown(index=False)
        else:
            return df.to_string()

class ContactManager:
    """Manages contact information"""
    
    def __init__(self, context: DifyExtractorContext, storage_path: str = '.dify_contacts'):
        self.context = context
        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(exist_ok=True)
    
    def store_contact(self, contact_id: str, contact_data: Dict[str, Any]) -> Dict[str, Any]:
        """Store contact information"""
        try:
            contact_file = self.storage_path / f"{contact_id}.json"
            contact_data['stored_at'] = datetime.now().isoformat()
            
            with open(contact_file, 'w') as f:
                json.dump(contact_data, f, indent=2)
            
            self.context.contacts_registry[contact_id] = contact_data
            
            return {
                'status': 'success',
                'message': f"Contact {contact_id} stored successfully",
                'contact_id': contact_id
            }
        except Exception as e:
            return {'status': 'error', 'error': str(e)}
    
    def retrieve_contact(self, contact_id: str) -> Dict[str, Any]:
        """Retrieve contact information"""
        try:
            contact_file = self.storage_path / f"{contact_id}.json"
            if contact_file.exists():
                with open(contact_file, 'r') as f:
                    return {'status': 'success', 'data': json.load(f)}
            else:
                return {'status': 'error', 'error': f"Contact {contact_id} not found"}
        except Exception as e:
            return {'status': 'error', 'error': str(e)}
    
    def list_contacts(self, search_query: Optional[str] = None) -> Dict[str, Any]:
        """List all stored contacts"""
        try:
            contacts = []
            for contact_file in self.storage_path.glob("*.json"):
                with open(contact_file, 'r') as f:
                    contact = json.load(f)
                    if search_query:
                        if any(search_query.lower() in str(v).lower() for v in contact.values()):
                            contacts.append(contact)
                    else:
                        contacts.append(contact)
            
            return {
                'status': 'success',
                'count': len(contacts),
                'contacts': contacts
            }
        except Exception as e:
            return {'status': 'error', 'error': str(e)}

def main():
    parser = argparse.ArgumentParser(description='Dify Document Extractor')
    parser.add_argument('--input', help='Input file path')
    parser.add_argument('--output', default='json', help='Output format: json|dict|markdown')
    parser.add_argument('--preserve-contacts', action='store_true', help='Preserve contact information')
    parser.add_argument('--sheet-name', help='Excel sheet name')
    parser.add_argument('--mode', default='text', help='PDF extraction mode: text|table|both')
    parser.add_argument('--action', help='Contact action: store|retrieve|update|delete|list')
    parser.add_argument('--contact-id', help='Contact identifier')
    parser.add_argument('--contact-data', help='Contact data (JSON format)')
    parser.add_argument('--search-query', help='Search query for contacts')
    parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    context = DifyExtractorContext()
    extractor = DocumentExtractor(context, verbose=args.verbose)
    contact_manager = ContactManager(context)
    
    try:
        if args.input:
            if args.input.endswith('.csv'):
                result = extractor.extract_csv(args.input, args.output, args.preserve_contacts)
            elif args.input.endswith(('.xlsx', '.xls')):
                result = extractor.extract_excel(args.input, args.sheet_name, args.output, args.preserve_contacts)
            elif args.input.endswith('.pdf'):
                result = extractor.extract_pdf(args.input, args.mode, args.preserve_contacts)
            else:
                result = {'status': 'error', 'error': 'Unsupported file format'}
            
            print(json.dumps(result, indent=2, default=str))
        
        elif args.action == 'store' and args.contact_id and args.contact_data:
            contact_data = json.loads(args.contact_data)
            result = contact_manager.store_contact(args.contact_id, contact_data)
            print(json.dumps(result, indent=2))
        
        elif args.action == 'retrieve' and args.contact_id:
            result = contact_manager.retrieve_contact(args.contact_id)
            print(json.dumps(result, indent=2, default=str))
        
        elif args.action == 'list':
            result = contact_manager.list_contacts(args.search_query)
            print(json.dumps(result, indent=2, default=str))
    
    except Exception as e:
        print(json.dumps({
            'status': 'error',
            'error': str(e),
            'context': context.to_dict()
        }, indent=2, default=str))
        sys.exit(1)

if __name__ == '__main__':
    main()
