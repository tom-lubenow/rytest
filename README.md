# rytest

A Rust-powered test collector for pytest that aims to significantly improve collection performance for large test suites.

## Features

- Speed

## Installation

```bash
pip install rytest
```

## Usage

Once installed, you can enable the Rust collector by using the `-p` flag:

```bash
pytest -p rytest.collect
```

Or add it to your pytest configuration in `conftest.py` or `pyproject.toml`:

```python
# conftest.py
pytest_plugins = ["rytest.collect"]
```

```toml
# pyproject.toml
[tool.pytest.ini_options]
addopts = "-p rytest.collect"
```

## Development

### Prerequisites

- Rust (latest stable version)
- Python 3.8+
- maturin
- pytest

### Setup

1. Clone the repository:
```bash
git clone https://github.com/tom-lubenow/rytest.git
cd rytest
```

2. Create and activate a virtual environment:
```bash
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
```

3. Install development dependencies:
```bash
pip install maturin pytest
```

4. Build and install in development mode:
```bash
maturin develop
```

### Running Tests

```bash
pytest
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The pytest team for creating an excellent testing framework
- The PyO3 team for making Rust-Python interop seamless 