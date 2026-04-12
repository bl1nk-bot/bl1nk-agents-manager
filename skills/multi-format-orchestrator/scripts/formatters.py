"""Output formatting utilities"""


class Formatter:
    """Handles output formatting and pretty-printing"""
    
    @staticmethod
    def normalize_whitespace(text: str) -> str:
        """Normalize whitespace in text"""
        return " ".join(text.split())
    
    @staticmethod
    def indent_text(text: str, indent_size: int = 2, level: int = 0) -> str:
        """Add indentation to text"""
        indent = " " * (indent_size * level)
        return "\n".join(f"{indent}{line}" for line in text.split("\n"))
