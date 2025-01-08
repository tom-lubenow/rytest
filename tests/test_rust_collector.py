"""Test the Rust collector implementation."""
import pytest

pytestmark = pytest.mark.rust_collector

def test_simple():
    """Simple test to verify collection."""
    assert True

class TestClass:
    """Test class to verify class collection."""
    
    def test_method(self):
        """Test method to verify method collection."""
        assert True
    
    def not_a_test(self):
        """Not a test method, should not be collected."""
        pass

@pytest.mark.parametrize("value", [1, 2, 3])
def test_parametrized(value):
    """Parametrized test to verify parametrization collection."""
    assert value > 0 