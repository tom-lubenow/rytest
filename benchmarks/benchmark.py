#!/usr/bin/env -S uv run
"""Benchmark script for comparing pytest collectors.

Dependencies:
pytest>=7.0.0
"""

import subprocess
import time
import re
import tempfile
import shutil
from pathlib import Path
from typing import Tuple, Set


def run_collection(target_dir: str, use_rytest: bool = False) -> Tuple[float, int, Set[str]]:
    """Run test collection and return time taken, number of tests, and test names.
    
    Returns:
        Tuple of (time_taken, num_tests, test_names)
    """
    cmd = ["pytest", "--collect-only", "-q", target_dir]
    if use_rytest:
        cmd.append("-p")
        cmd.append("rytest.collect")
    
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    end_time = time.time()
    
    # Extract number of tests from output
    match = re.search(r"(\d+) tests collected", result.stdout)
    num_tests = int(match.group(1)) if match else 0
    
    # Extract test names from output
    test_names = {line.strip() for line in result.stdout.splitlines() if "::" in line}
    
    return end_time - start_time, num_tests, test_names


def format_time(seconds: float) -> str:
    """Format time in seconds to a human readable string."""
    if seconds < 1:
        return f"{seconds*1000:.2f}ms"
    return f"{seconds:.2f}s"


def main():
    """Run the benchmark comparison."""
    script_dir = Path(__file__).parent
    source_dir = script_dir / "test_payload"
    
    # Create temporary directory for test files
    with tempfile.TemporaryDirectory() as temp_dir:
        target_dir = Path(temp_dir) / "test_payload_large"
        
        # Run multiply.py to create test files
        print("Creating test files...")
        subprocess.run([
            "python", str(script_dir / "multiply.py"), "10",
            "--source", str(source_dir),
            "--target", str(target_dir)
        ], check=True)
        
        # Run collection multiple times and average
        num_runs = 3
        default_times = []
        rytest_times = []
        num_tests = 0
        default_tests = set()
        rytest_tests = set()
        
        print("\nRunning collection benchmark...")
        print("================================")
        
        # Default collector
        print("\nDefault collector:")
        for i in range(num_runs):
            time_taken, tests, test_names = run_collection(str(target_dir), use_rytest=False)
            default_times.append(time_taken)
            num_tests = tests
            default_tests = test_names  # Save test names from last run
            print(f"  Run {i+1}: {format_time(time_taken)}")
        
        # Rytest collector
        print("\nRytest collector:")
        for i in range(num_runs):
            time_taken, tests, test_names = run_collection(str(target_dir), use_rytest=True)
            rytest_times.append(time_taken)
            rytest_tests = test_names  # Save test names from last run
            print(f"  Run {i+1}: {format_time(time_taken)}")
        
        # Calculate averages
        default_avg = sum(default_times) / len(default_times)
        rytest_avg = sum(rytest_times) / len(rytest_times)
        
        # Compare test outputs
        only_in_default = default_tests - rytest_tests
        only_in_rytest = rytest_tests - default_tests
        
        # Print results
        print("\nResults:")
        print("========")
        print(f"Number of tests: {num_tests}")
        print(f"Default collector average: {format_time(default_avg)}")
        print(f"Rytest collector average:  {format_time(rytest_avg)}")
        print(f"\nSpeedup: {default_avg/rytest_avg:.2f}x")
        
        # Print test differences if any
        if only_in_default or only_in_rytest:
            print("\nTest differences found!")
            if only_in_default:
                print("\nTests only in default collector:")
                for test in sorted(only_in_default):
                    print(f"  {test}")
            if only_in_rytest:
                print("\nTests only in rytest collector:")
                for test in sorted(only_in_rytest):
                    print(f"  {test}")
        else:
            print("\nBoth collectors found the exact same tests.")


if __name__ == "__main__":
    main() 