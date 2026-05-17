"""
Unit tests for TracciaMemoryNotary adapter.
"""

import unittest
import tempfile
import json
import os
import hashlib
from pathlib import Path
import sys

# Add the parent directory to sys.path to import the module
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from erc.traccia_adapter import TracciaMemoryNotary


class TestTracciaMemoryNotary(unittest.TestCase):
    """Test cases for TracciaMemoryNotary."""
    
    def setUp(self):
        """Set up test fixtures."""
        # Create temporary directories
        self.temp_dir = tempfile.mkdtemp()
        self.memory_dir = Path(self.temp_dir) / "memory"
        self.receipt_dir = Path(self.temp_dir) / "receipts"
        self.memory_dir.mkdir(parents=True, exist_ok=True)
        self.receipt_dir.mkdir(parents=True, exist_ok=True)
        
        # Create test memory file
        self.memory_file = self.memory_dir / "memory.json"
        self.test_memories = [
            {
                "id": "test-uuid-1",
                "type": "preference",
                "scope": "coding",
                "content": "用户偏好极简架构",
                "source_event_ids": ["evt-001"],
                "created_at": "2026-05-17T00:00:00Z"
            },
            {
                "id": "test-uuid-2",
                "type": "project_context",
                "scope": "coding",
                "content": "项目使用Python 3.11",
                "source_event_ids": ["evt-002", "evt-003"],
                "created_at": "2026-05-17T00:00:00Z"
            }
        ]
        
        with open(self.memory_file, 'w', encoding='utf-8') as f:
            json.dump(self.test_memories, f, ensure_ascii=False)
    
    def tearDown(self):
        """Clean up temporary files."""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_notarize_all_returns_two_receipts(self):
        """Test that notarize_all returns two receipt paths."""
        notary = TracciaMemoryNotary(
            memory_path=str(self.memory_file),
            receipt_dir=str(self.receipt_dir)
        )
        
        receipt_paths = notary.notarize_all()
        
        self.assertEqual(len(receipt_paths), 2)
        for path in receipt_paths:
            self.assertTrue(os.path.exists(path))
    
    def test_receipt_contains_required_fields(self):
        """Test that each receipt contains all required fields."""
        notary = TracciaMemoryNotary(
            memory_path=str(self.memory_file),
            receipt_dir=str(self.receipt_dir)
        )
        
        receipt_paths = notary.notarize_all()
        
        required_fields = [
            "receipt_id", "memory_id", "memory_type", "scope",
            "content_hash", "source_events", "issued_at", "issuer", "signature"
        ]
        
        for path in receipt_paths:
            with open(path, 'r', encoding='utf-8') as f:
                receipt = json.load(f)
            
            for field in required_fields:
                self.assertIn(field, receipt)
    
    def test_content_hash_is_correct(self):
        """Test that content_hash matches SHA-256 of memory content."""
        notary = TracciaMemoryNotary(
            memory_path=str(self.memory_file),
            receipt_dir=str(self.receipt_dir)
        )
        
        receipt_paths = notary.notarize_all()
        
        # Load memories again to verify hash
        with open(self.memory_file, 'r', encoding='utf-8') as f:
            memories = json.load(f)
        
        # Create a mapping from memory_id to content
        memory_content_map = {m["id"]: m["content"] for m in memories}
        
        for path in receipt_paths:
            with open(path, 'r', encoding='utf-8') as f:
                receipt = json.load(f)
            
            memory_id = receipt["memory_id"]
            expected_content = memory_content_map[memory_id]
            expected_hash = hashlib.sha256(expected_content.encode('utf-8')).hexdigest()
            
            self.assertEqual(receipt["content_hash"], expected_hash)
    
    def test_notarize_memory_single(self):
        """Test notarizing a single memory."""
        notary = TracciaMemoryNotary(
            memory_path=str(self.memory_file),
            receipt_dir=str(self.receipt_dir)
        )
        
        memory = self.test_memories[0]
        receipt_path = notary.notarize_memory(memory)
        
        self.assertTrue(os.path.exists(receipt_path))
        
        with open(receipt_path, 'r', encoding='utf-8') as f:
            receipt = json.load(f)
        
        self.assertEqual(receipt["memory_id"], memory["id"])
        self.assertEqual(receipt["memory_type"], memory["type"])
        self.assertEqual(receipt["scope"], memory["scope"])
        self.assertEqual(receipt["source_events"], memory["source_event_ids"])
        self.assertEqual(receipt["issuer"], "ERC Traccia Adapter v1.0")
        self.assertEqual(receipt["signature"], "unsigned")
    
    def test_empty_memory_file(self):
        """Test behavior with empty memory file."""
        empty_file = self.memory_dir / "empty.json"
        with open(empty_file, 'w', encoding='utf-8') as f:
            json.dump([], f)
        
        notary = TracciaMemoryNotary(
            memory_path=str(empty_file),
            receipt_dir=str(self.receipt_dir)
        )
        
        receipt_paths = notary.notarize_all()
        self.assertEqual(len(receipt_paths), 0)
    
    def test_nonexistent_memory_file(self):
        """Test behavior with non-existent memory file."""
        nonexistent_file = self.memory_dir / "nonexistent.json"
        
        notary = TracciaMemoryNotary(
            memory_path=str(nonexistent_file),
            receipt_dir=str(self.receipt_dir)
        )
        
        receipt_paths = notary.notarize_all()
        self.assertEqual(len(receipt_paths), 0)


if __name__ == '__main__':
    unittest.main()