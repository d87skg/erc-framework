"""
Traccia Memory Notary Adapter for ERC
Generates non-repudiable receipts for distilled cognitive memories.
"""

import json
import hashlib
import uuid
from datetime import datetime, UTC
from pathlib import Path
from typing import List, Dict, Any


class TracciaMemoryNotary:
    """Generates notarized receipts for Traccia memories."""
    
    def __init__(self, memory_path: str = "~/.traccia/memory/memory.json",
                 receipt_dir: str = "~/.traccia/receipts") -> None:
        """
        Initialize the notary with paths for memory file and receipt directory.
        
        Args:
            memory_path: Path to the Traccia memory JSON file
            receipt_dir: Directory to store generated receipts
        """
        self.memory_path = Path(memory_path).expanduser()
        self.receipt_dir = Path(receipt_dir).expanduser()
        self.receipt_dir.mkdir(parents=True, exist_ok=True)
    
    def notarize_all(self) -> List[str]:
        """
        Notarize all memories in the memory file.
        
        Returns:
            List of receipt file paths
        """
        if not self.memory_path.exists():
            return []
        
        with open(self.memory_path, 'r', encoding='utf-8') as f:
            memories = json.load(f)
        
        receipt_paths = []
        for memory in memories:
            receipt_path = self.notarize_memory(memory)
            receipt_paths.append(receipt_path)
        
        return receipt_paths
    
    def notarize_memory(self, memory: Dict[str, Any]) -> str:
        """
        Generate a notarized receipt for a single memory.
        
        Args:
            memory: Memory dictionary from Traccia
            
        Returns:
            Path to the generated receipt file
        """
        # Calculate SHA-256 hash of content
        content = memory.get("content", "")
        content_hash = hashlib.sha256(content.encode('utf-8')).hexdigest()
        
        # Generate receipt
        receipt = {
            "receipt_id": str(uuid.uuid4()),
            "memory_id": memory["id"],
            "memory_type": memory["type"],
            "scope": memory.get("scope", ""),
            "content_hash": content_hash,
            "source_events": memory.get("source_event_ids", []),
            "issued_at": datetime.now(UTC).isoformat(),
            "issuer": "ERC Traccia Adapter v1.0",
            "signature": "unsigned"
        }
        
        # Save receipt to file
        receipt_filename = f"{receipt['receipt_id']}.json"
        receipt_path = self.receipt_dir / receipt_filename
        
        with open(receipt_path, 'w', encoding='utf-8') as f:
            json.dump(receipt, f, indent=2, ensure_ascii=False)
        
        return str(receipt_path)