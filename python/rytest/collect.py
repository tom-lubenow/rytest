"""Fast pytest collector plugin implemented in Rust."""
import pytest
from pathlib import Path

import rytest_core


def pytest_configure(config):
    config.pluginmanager.register(RytestCollector(config))


class RytestCollector:
    """Plugin that replaces pytest's default collection with a Rust implementation."""

    def __init__(self, config):
        self.config = config
        self.collector = rytest_core.Collector(config)

    @staticmethod
    @pytest.hookimpl(tryfirst=True)
    def pytest_collect_file(file_path=None, path=None, parent=None):
        """Hook into pytest's file collection process."""
        if file_path is None or parent is None:
            return None

        if not str(file_path).endswith('.py'):
            return None

        # Create a new collector instance for this file
        collector = rytest_core.Collector(parent.config)
        result = collector.pytest_collect_file(str(file_path), parent)
        return result if result is not None else None

    @staticmethod
    @pytest.hookimpl(tryfirst=True)
    def pytest_collect_directory(path=None, parent=None):
        """Hook into pytest's directory collection process."""
        if path is None or parent is None:
            # If path or parent is not provided, skip collection
            return None

        # Create a new collector instance for this directory
        collector = rytest_core.Collector(parent.config)
        result = collector.pytest_collect_directory(str(path), parent)
        return result if result is not None else None
