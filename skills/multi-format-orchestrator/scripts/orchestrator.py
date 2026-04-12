#!/usr/bin/env python3
"""
Multi-Format Orchestrator
Universal data format handler for JSON, YAML, TOML, XML, and OpenAPI
Version: 2.0.0
"""

import json
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))

from format_handlers import (
    JSONHandler, YAMLHandler, TOMLHandler, XMLHandler, OpenAPIHandler,
    FormatHandler, FormatType
)
from validators import SchemaValidator, SecurityScanner
from formatters import Formatter

@dataclass
class TransformResult:
    """Result of transformation operation"""
    formatted: str
    valid: bool
    warnings: List[str]
    errors: List[str]
    metadata: Dict[str, Any]
    source_format: str
    target_format: str
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return asdict(self)
    
    def to_json(self) -> str:
        """Convert to JSON string"""
        return json.dumps(self.to_dict(), indent=2)


class MultiFormatOrchestrator:
    """Main orchestrator for multi-format data handling"""
    
    # Handler registry
    HANDLERS: Dict[FormatType, type] = {
        FormatType.JSON: JSONHandler,
        FormatType.YAML: YAMLHandler,
        FormatType.TOML: TOMLHandler,
        FormatType.XML: XMLHandler,
        FormatType.OPENAPI: OpenAPIHandler,
    }
    
    def __init__(self, strict_mode: bool = False):
        """Initialize orchestrator
        
        Args:
            strict_mode: Treat warnings as errors
        """
        self.strict_mode = strict_mode
        self.handlers = {fmt: handler() for fmt, handler in self.HANDLERS.items()}
        self.validator = SchemaValidator()
        self.security_scanner = SecurityScanner()
        self.formatter = Formatter()
    
    def detect_format(self, data: str) -> Tuple[FormatType, float]:
        """Detect input format and confidence level
        
        Args:
            data: Input data string
            
        Returns:
            Tuple of (detected_format, confidence_score)
        """
        scores = {}
        
        for fmt, handler in self.handlers.items():
            try:
                handler.parse(data)
                scores[fmt] = 1.0  # Successfully parsed
            except Exception as e:
                # Calculate confidence based on error type
                error_str = str(e).lower()
                if "json" in error_str or fmt == FormatType.JSON:
                    scores[fmt] = 0.1
                elif "yaml" in error_str or fmt == FormatType.YAML:
                    scores[fmt] = 0.2
                else:
                    scores[fmt] = 0.0
        
        if not scores or max(scores.values()) == 0:
            raise ValueError(f"Unable to detect format from input")
        
        detected_fmt = max(scores, key=scores.get)
        confidence = scores[detected_fmt]
        
        return detected_fmt, confidence
    
    def transform(
        self,
        data: str,
        source_format: Optional[str] = None,
        target_format: str = "json",
        validate: bool = True,
        pretty_print: bool = True,
        indent: int = 2,
    ) -> TransformResult:
        """Transform data between formats
        
        Args:
            data: Input data string
            source_format: Source format name (auto-detect if None)
            target_format: Target format name
            validate: Perform validation
            pretty_print: Pretty-print output
            indent: Indentation level
            
        Returns:
            TransformResult with formatted data and validation info
        """
        warnings = []
        errors = []
        
        try:
            # Detect format if not specified
            if source_format is None:
                detected_fmt, confidence = self.detect_format(data)
                if confidence < 0.8:
                    warnings.append(
                        f"Format detection confidence low ({confidence:.0%}). "
                        f"Detected: {detected_fmt.value}"
                    )
                source_fmt = detected_fmt
            else:
                try:
                    source_fmt = FormatType(source_format.lower())
                except ValueError:
                    errors.append(f"Unknown source format: {source_format}")
                    return TransformResult(
                        formatted="",
                        valid=False,
                        warnings=warnings,
                        errors=errors,
                        metadata={},
                        source_format=source_format or "unknown",
                        target_format=target_format,
                    )
            
            # Get target format
            try:
                target_fmt = FormatType(target_format.lower())
            except ValueError:
                errors.append(f"Unknown target format: {target_format}")
                return TransformResult(
                    formatted="",
                    valid=False,
                    warnings=warnings,
                    errors=errors,
                    metadata={},
                    source_format=str(source_fmt.value),
                    target_format=target_format,
                )
            
            # Parse input
            source_handler = self.handlers[source_fmt]
            parsed_data = source_handler.parse(data)
            
            # Validate if requested
            metadata = {}
            is_valid = True
            
            if validate:
                validation_result = self.validator.validate(
                    parsed_data, source_fmt
                )
                is_valid = validation_result.get("valid", True)
                warnings.extend(validation_result.get("warnings", []))
                errors.extend(validation_result.get("errors", []))
                metadata.update(validation_result.get("metadata", {}))
            
            # Security scan
            security_result = self.security_scanner.scan(parsed_data)
            if security_result["issues"]:
                warnings.extend(security_result["issues"])
            
            # Transform to target format if different
            if source_fmt != target_fmt:
                # Potential data loss warning for some transformations
                if not self._is_lossless_conversion(source_fmt, target_fmt):
                    warnings.append(
                        f"Conversion {source_fmt.value}→{target_fmt.value} "
                        "may result in data loss"
                    )
            
            # Format output
            target_handler = self.handlers[target_fmt]
            formatted = target_handler.format(
                parsed_data,
                pretty_print=pretty_print,
                indent=indent
            )
            
            return TransformResult(
                formatted=formatted,
                valid=is_valid and len(errors) == 0,
                warnings=warnings,
                errors=errors,
                metadata=metadata,
                source_format=source_fmt.value,
                target_format=target_fmt.value,
            )
            
        except Exception as e:
            errors.append(f"Transform error: {str(e)}")
            return TransformResult(
                formatted="",
                valid=False,
                warnings=warnings,
                errors=errors,
                metadata={},
                source_format=source_format or "unknown",
                target_format=target_format,
            )
    
    def validate_only(
        self,
        data: str,
        format_type: str,
        strict: Optional[bool] = None,
    ) -> Dict[str, Any]:
        """Validate data without transformation
        
        Args:
            data: Data to validate
            format_type: Format to validate against
            strict: Override strict mode
            
        Returns:
            Validation result dictionary
        """
        try:
            fmt = FormatType(format_type.lower())
            handler = self.handlers[fmt]
            parsed = handler.parse(data)
            
            result = self.validator.validate(parsed, fmt)
            
            # Check strict mode
            is_strict = strict if strict is not None else self.strict_mode
            if is_strict and result.get("warnings"):
                result["valid"] = False
                result["errors"].extend(result.get("warnings", []))
            
            return result
            
        except Exception as e:
            return {
                "valid": False,
                "errors": [f"Validation error: {str(e)}"],
                "warnings": [],
                "metadata": {}
            }
    
    @staticmethod
    def _is_lossless_conversion(from_fmt: FormatType, to_fmt: FormatType) -> bool:
        """Check if conversion is lossless"""
        # Most conversions preserve data when properly structured
        # Only warn about specific problematic conversions
        problematic = {
            (FormatType.XML, FormatType.JSON),  # XML attributes lost
            (FormatType.XML, FormatType.YAML),  # XML namespaces may be lost
        }
        return (from_fmt, to_fmt) not in problematic


def main():
    """CLI entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Multi-Format Orchestrator - Universal data format handler"
    )
    parser.add_argument("action", choices=["transform", "validate", "detect"],
                       help="Action to perform")
    parser.add_argument("--data", type=str, help="Input data (or read from stdin)")
    parser.add_argument("--from", dest="source_format",
                       help="Source format (auto-detect if omitted)")
    parser.add_argument("--to", dest="target_format", default="json",
                       help="Target format (default: json)")
    parser.add_argument("--pretty", action="store_true", default=True,
                       help="Pretty-print output")
    parser.add_argument("--indent", type=int, default=2,
                       help="Indentation level")
    parser.add_argument("--strict", action="store_true",
                       help="Strict validation mode")
    
    args = parser.parse_args()
    
    # Read data
    if args.data:
        data = args.data
    else:
        data = sys.stdin.read()
    
    orchestrator = MultiFormatOrchestrator(strict_mode=args.strict)
    
    if args.action == "transform":
        result = orchestrator.transform(
            data,
            source_format=args.source_format,
            target_format=args.target_format,
            pretty_print=args.pretty,
            indent=args.indent,
        )
        print(result.to_json())
        
    elif args.action == "validate":
        if not args.source_format:
            fmt, conf = orchestrator.detect_format(data)
            args.source_format = fmt.value
        
        result = orchestrator.validate_only(
            data,
            args.source_format,
            strict=args.strict
        )
        print(json.dumps(result, indent=2))
        
    elif args.action == "detect":
        fmt, confidence = orchestrator.detect_format(data)
        result = {
            "format": fmt.value,
            "confidence": f"{confidence:.0%}",
            "message": f"Detected format: {fmt.value} ({confidence:.0%} confidence)"
        }
        print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
