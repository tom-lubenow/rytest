# rytest

A Rust-powered test collector for pytest that aims to significantly improve collection performance for large test suites.

## Features

- Speed

## Project status

This project is currently in the early stages of development. It is not yet ready for production use.
It appears to be passing the pytest repo's collection tests.

It is currently not any faster.

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

The project includes a comprehensive test suite that runs against pytest's own collection tests to ensure compatibility and correctness.

### Prerequisites

- Rust (latest stable version)
- Python 3.8+
- uv (will be installed automatically by the check script)

### Setup

1. Clone the repository with submodules:
```bash
git clone --recursive https://github.com/tom-lubenow/rytest.git
cd rytest
```

2. Run the check script to set up the environment and run tests:
```bash
./scripts/check.sh
```

This will:
- Create a virtual environment
- Install all dependencies using uv
- Build the Rust collector
- Run the test suite

### Manual Setup

If you prefer to set things up manually:

1. Create and activate a virtual environment:
```bash
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
```

2. Install dependencies:
```bash
uv pip install pytest maturin
```

3. Build and install in development mode:
```bash
maturin develop
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Make sure to run the test suite using `./scripts/check.sh` before submitting.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The pytest team for creating an excellent testing framework
- The PyO3 team for making Rust-Python interop seamless 