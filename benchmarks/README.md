This module will provide one or more nominally-sized pytest projects. We then use a script to arbitrarily multiply the size of the test suite in order to benchmark the performance of the rytest test collector against the default pytest implementation.

The benchmark.py script also compares the collection output to ensure integrity.

## Usage

```bash
./benchmark.py
```

## Dependencies

- uv