"""Fast pytest collector plugin implemented in Rust."""
import pytest
from pathlib import Path

from . import rytest_core


def pytest_configure(config):
    config.pluginmanager.register(RytestCollector(config))


class RytestCollector:
    """Plugin that replaces pytest's default collection with a Rust implementation."""

    def __init__(self, config):
        self.config = config
        self.collector = rytest_core.Collector(config)

    @pytest.hookimpl(tryfirst=True)
    def pytest_collect_file(self, file_path, parent):
        """Hook into pytest's file collection process."""
        if not str(file_path).endswith('.py'):
            return None

        result = self.collector.pytest_collect_file(str(file_path), parent)
        return result if result is not None else []

    @pytest.hookimpl(tryfirst=True)
    def pytest_collect_directory(self, path, parent):
        """Hook into pytest's directory collection process."""
        if isinstance(path, str):
            path = Path(path)

        result = self.collector.pytest_collect_directory(str(path), parent)
        return result if result is not None else []
