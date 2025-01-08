"""Test file demonstrating fixtures and indirect parameterization."""
import pytest
from typing import Generator


@pytest.fixture(params=[f"db_{i}" for i in range(10)])
def database(request):
    """Fixture that provides different database configurations."""
    return request.param


@pytest.fixture(params=[f"user_{i}" for i in range(20)])
def user(request):
    """Fixture that provides different user configurations."""
    return request.param


@pytest.fixture
def complex_setup(database, user) -> Generator[dict, None, None]:
    """Fixture that depends on other parameterized fixtures."""
    yield {"db": database, "user": user}


# Tests using fixtures directly
@pytest.mark.parametrize("operation", [
    f"operation_{i}" for i in range(50)
])
def test_with_fixtures(database, user, operation):
    """Test using multiple fixtures with parameterization."""
    pass


# Tests with indirect parameterization
@pytest.mark.parametrize("database", [f"special_db_{i}" for i in range(30)], indirect=True)
@pytest.mark.parametrize("user", [f"special_user_{i}" for i in range(20)], indirect=True)
def test_indirect(database, user):
    """Test using indirect parameterization of fixtures."""
    pass


class TestWithFixtures:
    """Class demonstrating fixture usage in methods."""
    
    @pytest.fixture(autouse=True)
    def setup(self):
        """Setup fixture for all tests in the class."""
        pass
    
    @pytest.mark.parametrize("scenario", range(100))
    def test_with_autouse(self, scenario, complex_setup):
        """Test using autouse fixture and complex setup."""
        pass
    
    @pytest.mark.parametrize("database", [f"method_db_{i}" for i in range(10)], indirect=True)
    def test_method_specific(self, database):
        """Test with method-specific indirect parameterization."""
        pass


# Create parameterized test classes
for i in range(5):
    class_name = f"TestFixtureClass{i}"
    class_dict = {
        "__module__": __name__,
        "setup_class": pytest.fixture(autouse=True)(lambda self: None),
    }
    
    # Add test methods
    for j in range(10):
        method_name = f"test_fixture_{j}"
        
        @pytest.mark.parametrize("user", [f"dynamic_user_{k}" for k in range(10)], indirect=True)
        def test_method(self, user, j=j):
            """Dynamically created test method."""
            pass
        
        test_method.__name__ = method_name
        class_dict[method_name] = test_method
    
    # Create the class
    globals()[class_name] = type(class_name, (object,), class_dict) 