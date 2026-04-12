"""Format handlers for all supported data formats"""

import json
import xml.etree.ElementTree as ET
from abc import ABC, abstractmethod
from enum import Enum
from typing import Any, Dict

class FormatType(Enum):
    """Supported data formats"""
    JSON = "json"
    YAML = "yaml"
    TOML = "toml"
    XML = "xml"
    OPENAPI = "openapi"


class FormatHandler(ABC):
    """Abstract base class for format handlers"""
    
    @abstractmethod
    def parse(self, data: str) -> Any:
        """Parse string to internal representation"""
        pass
    
    @abstractmethod
    def format(self, data: Any, pretty_print: bool = True, indent: int = 2) -> str:
        """Format internal representation to string"""
        pass


class JSONHandler(FormatHandler):
    """Handler for JSON format"""
    
    def parse(self, data: str) -> Any:
        """Parse JSON string"""
        return json.loads(data)
    
    def format(self, data: Any, pretty_print: bool = True, indent: int = 2) -> str:
        """Format to JSON string"""
        if pretty_print:
            return json.dumps(data, indent=indent, ensure_ascii=False)
        else:
            return json.dumps(data, separators=(',', ':'), ensure_ascii=False)


class YAMLHandler(FormatHandler):
    """Handler for YAML format"""
    
    def parse(self, data: str) -> Any:
        """Parse YAML string"""
        try:
            import yaml
            return yaml.safe_load(data)
        except ImportError:
            raise ImportError("pyyaml not installed. Install with: pip install pyyaml")
    
    def format(self, data: Any, pretty_print: bool = True, indent: int = 2) -> str:
        """Format to YAML string"""
        try:
            import yaml
            class CustomDumper(yaml.SafeDumper):
                pass
            
            def dict_representer(dumper, data):
                return dumper.represent_mapping('tag:yaml.org,2002:map', data.items())
            
            CustomDumper.add_representer(dict, dict_representer)
            
            return yaml.dump(data, Dumper=CustomDumper, default_flow_style=False)
        except ImportError:
            raise ImportError("pyyaml not installed")


class TOMLHandler(FormatHandler):
    """Handler for TOML format"""
    
    def parse(self, data: str) -> Any:
        """Parse TOML string"""
        try:
            import tomllib
        except ImportError:
            try:
                import tomli as tomllib
            except ImportError:
                raise ImportError("tomli not installed. Install with: pip install tomli")
        
        return tomllib.loads(data)
    
    def format(self, data: Any, pretty_print: bool = True, indent: int = 2) -> str:
        """Format to TOML string"""
        try:
            import tomli_w
            return tomli_w.dumps(data)
        except ImportError:
            raise ImportError("tomli_w not installed. Install with: pip install tomli_w")


class XMLHandler(FormatHandler):
    """Handler for XML format"""
    
    def parse(self, data: str) -> Dict[str, Any]:
        """Parse XML string to dict"""
        root = ET.fromstring(data)
        return {root.tag: self._xml_to_dict(root)}
    
    def format(self, data: Dict[str, Any], pretty_print: bool = True, indent: int = 2) -> str:
        """Format dict to XML string"""
        # Get root element name
        root_name = list(data.keys())[0] if isinstance(data, dict) else "root"
        root_data = data[root_name] if isinstance(data, dict) else data
        
        root = self._dict_to_xml(root_name, root_data)
        
        if pretty_print:
            self._indent_xml(root, 0, indent)
        
        return ET.tostring(root, encoding='unicode')
    
    @staticmethod
    def _xml_to_dict(element):
        """Convert XML element to dict"""
        result = element.attrib.copy()
        
        if element.text and element.text.strip():
            if result:
                result['text'] = element.text.strip()
            else:
                return element.text.strip()
        
        children = {}
        for child in element:
            child_data = XMLHandler._xml_to_dict(child)
            if child.tag in children:
                if not isinstance(children[child.tag], list):
                    children[child.tag] = [children[child.tag]]
                children[child.tag].append(child_data)
            else:
                children[child.tag] = child_data
        
        result.update(children)
        return result if result else None
    
    @staticmethod
    def _dict_to_xml(tag, data):
        """Convert dict to XML element"""
        element = ET.Element(tag)
        
        if isinstance(data, dict):
            for key, value in data.items():
                if key == 'text':
                    element.text = str(value)
                elif key.startswith('@'):
                    element.set(key[1:], str(value))
                else:
                    if isinstance(value, list):
                        for item in value:
                            element.append(XMLHandler._dict_to_xml(key, item))
                    else:
                        element.append(XMLHandler._dict_to_xml(key, value))
        elif data is not None:
            element.text = str(data)
        
        return element
    
    @staticmethod
    def _indent_xml(elem, level, indent_size):
        """Add pretty-printing to XML element"""
        indent_str = " " * (level * indent_size)
        if len(elem):
            if not elem.text or not elem.text.strip():
                elem.text = f"\n{indent_str}  "
            if not elem.tail or not elem.tail.strip():
                elem.tail = f"\n{indent_str}"
            for i, child in enumerate(elem):
                XMLHandler._indent_xml(child, level + 1, indent_size)
                if i < len(elem) - 1:
                    child.tail = f"\n{indent_str}  "
                else:
                    child.tail = f"\n{indent_str}"
        else:
            if level and (not elem.tail or not elem.tail.strip()):
                elem.tail = f"\n{indent_str}"


class OpenAPIHandler(FormatHandler):
    """Handler for OpenAPI format (3.x)"""
    
    def parse(self, data: str) -> Any:
        """Parse OpenAPI spec (YAML or JSON)"""
        # Try JSON first
        try:
            return json.loads(data)
        except json.JSONDecodeError:
            # Try YAML
            try:
                import yaml
                return yaml.safe_load(data)
            except:
                raise ValueError("Invalid OpenAPI spec: must be valid JSON or YAML")
    
    def format(self, data: Any, pretty_print: bool = True, indent: int = 2) -> str:
        """Format to OpenAPI YAML format (recommended)"""
        try:
            import yaml
            if pretty_print:
                return yaml.dump(data, default_flow_style=False)
            else:
                return json.dumps(data, separators=(',', ':'))
        except ImportError:
            return json.dumps(data, indent=indent if pretty_print else None)
