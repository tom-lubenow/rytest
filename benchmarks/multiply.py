#!/usr/bin/env -S uv run

import shutil
import sys
import argparse
from pathlib import Path


def multiply_payload(source_dir: str, target_dir: str, multiplier: int) -> None:
    """Copy test files with unique names to create a larger test suite.
    
    Args:
        source_dir: Directory containing the original test files
        target_dir: Directory to write the multiplied files to
        multiplier: Number of copies to create
    """
    source_path = Path(source_dir)
    target_path = Path(target_dir)
    
    # Create target directory if it doesn't exist
    target_path.mkdir(parents=True, exist_ok=True)
    
    # Copy each test file multiple times
    for source_file in source_path.glob("test_*.py"):
        base_name = source_file.stem
        extension = source_file.suffix
        
        for i in range(multiplier):
            target_file = target_path / f"{base_name}_{i:03d}{extension}"
            shutil.copy2(source_file, target_file)
            print(f"Created {target_file.name}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Multiply test files for benchmarking")
    parser.add_argument("multiplier", type=int, help="Number of copies to create")
    parser.add_argument("--source", type=str, help="Source directory containing test files")
    parser.add_argument("--target", type=str, help="Target directory for multiplied files")
    
    args = parser.parse_args()
    
    if args.multiplier < 1:
        print("Error: MULTIPLIER must be positive")
        sys.exit(1)
    
    # Get directories
    script_dir = Path(__file__).parent
    source_dir = Path(args.source) if args.source else script_dir / "test_payload"
    target_dir = Path(args.target) if args.target else script_dir / "test_payload_large"
    
    # Remove existing target directory
    if target_dir.exists():
        shutil.rmtree(target_dir)
    
    print(f"Multiplying test files by {args.multiplier}...")
    multiply_payload(source_dir, target_dir, args.multiplier)
    print("Done!")


if __name__ == "__main__":
    main() 