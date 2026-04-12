"""Validation framework for all formats"""

import re
import json
from typing import Dict, Any, List
from format_handlers import FormatType


class SchemaValidator:
    """Validates data against format schemas"""
    
    def validate(self, data: Any, format_type: FormatType) -> Dict[str, Any]:
        """Validate data against format rules"""
        warnings = []
        errors = []
        metadata = {}
        
        try:
            if format_type == FormatType.JSON:
                errors, warnings = self._validate_json(data)
            elif format_type == FormatType.YAML:
                errors, warnings = self._validate_yaml(data)
            elif format_type == FormatType.TOML:
                errors, warnings = self._validate_toml(data)
            elif format_type == FormatType.XML:
                errors, warnings = self._validate_xml(data)
            elif format_type == FormatType.OPENAPI:
                errors, warnings = self._validate_openapi(data)
            
            metadata["format"] = format_type.value
            metadata["validation_version"] = "1.0"
            
            return {
                "valid": len(errors) == 0,
                "errors": errors,
                "warnings": warnings,
                "metadata": metadata
            }
        except Exception as e:
            return {
                "valid": False,
                "errors": [f"Validation exception: {str(e)}"],
                "warnings": [],
                "metadata": metadata
            }
    
    @staticmethod
    def _validate_json(data: Any) -> tuple:
        """Validate JSON structure"""
        errors = []
        warnings = []
        
        # Check for circular references (limit depth)
        def check_depth(obj, depth=0, max_depth=100):
            if depth > max_depth:
                errors.append(f"Nesting too deep (max {max_depth})")
            if isinstance(obj, dict):
                for v in obj.values():
                    check_depth(v, depth + 1, max_depth)
            elif isinstance(obj, list):
                for item in obj:
                    check_depth(item, depth + 1, max_depth)
        
        check_depth(data)
        return errors, warnings
    
    @staticmethod
    def _validate_yaml(data: Any) -> tuple:
        """Validate YAML structure"""
        # YAML is flexible, just check basic structure
        return [], []
    
    @staticmethod
    def _validate_toml(data: Any) -> tuple:
        """Validate TOML structure"""
        errors = []
        warnings = []
        # TOML requires root to be dict/table
        if not isinstance(data, dict):
            errors.append("TOML root must be a table (dictionary)")
        return errors, warnings
    
    @staticmethod
    def _validate_xml(data: Any) -> tuple:
        """Validate XML structure"""
        return [], []
    
    @staticmethod
    def _validate_openapi(data: Any) -> tuple:
        """Validate OpenAPI spec"""
        errors = []
        warnings = []
        
        if not isinstance(data, dict):
            errors.append("OpenAPI spec must be an object")
            return errors, warnings
        
        # Check required fields
        if "openapi" not in data:
            errors.append("Missing required field 'openapi'")
        if "info" not in data:
            errors.append("Missing required field 'info'")
        if "paths" not in data:
            warnings.append("No 'paths' defined in OpenAPI spec")
        
        # Check version format
        if "openapi" in data:
            version = data["openapi"]
            if not version.startswith("3."):
                errors.append(f"Unsupported OpenAPI version: {version}")
        
        return errors, warnings


class SecurityScanner:
    """Scans for security issues in data"""
    
    def scan(self, data: Any) -> Dict[str, List[str]]:
        """Scan for security issues"""
        issues = []
        
        # Scan for hardcoded secrets
        secrets_found = self._find_secrets(data)
        if secrets_found:
            issues.extend(secrets_found)
        
        return {"issues": issues}
    
    @staticmethod
    def _find_secrets(obj, path="", depth=0):
        """Find potential hardcoded secrets"""
        issues = []
        max_depth = 50
        
        if depth > max_depth:
            return issues
        
        if isinstance(obj, dict):
            for key, value in obj.items():
                new_path = f"{path}.{key}" if path else key
                
                # Check suspicious key names
                key_lower = key.lower()
                if any(x in key_lower for x in ["password", "secret", "token", "key", "api"]):
                    if isinstance(value, str) and len(value) > 10:
                        issues.append(
                            f"Potential secret at {new_path}: suspicious key name"
                        )
                
                # Recurse
                issues.extend(SecurityScanner._find_secrets(value, new_path, depth + 1))
        
        elif isinstance(obj, list):
            for i, item in enumerate(obj):
                new_path = f"{path}[{i}]"
                issues.extend(SecurityScanner._find_secrets(item, new_path, depth + 1))
        
        return issues
