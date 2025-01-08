"""Test file demonstrating class hierarchies."""
import pytest


class TestBase:
    """Base class with common test methods."""
    
    def test_base_1(self):
        pass
        
    def test_base_2(self):
        pass


class TestLevel1(TestBase):
    """First level of inheritance."""
    
    def test_level1_1(self):
        pass
        
    def test_level1_2(self):
        pass


class TestLevel2A(TestLevel1):
    """Second level of inheritance, branch A."""
    
    @pytest.mark.parametrize("i", range(50))
    def test_level2a(self, i):
        pass


class TestLevel2B(TestLevel1):
    """Second level of inheritance, branch B."""
    
    @pytest.mark.parametrize("i", range(50))
    def test_level2b(self, i):
        pass


class TestLevel3A(TestLevel2A):
    """Third level of inheritance, branch A."""
    
    def test_level3a_1(self):
        pass
        
    def test_level3a_2(self):
        pass
        
    @pytest.mark.parametrize("x", range(20))
    def test_level3a_param(self, x):
        pass


class TestLevel3B(TestLevel2B):
    """Third level of inheritance, branch B."""
    
    def test_level3b_1(self):
        pass
        
    def test_level3b_2(self):
        pass
        
    @pytest.mark.parametrize("x", range(20))
    def test_level3b_param(self, x):
        pass


# Create 10 concrete test classes
for i in range(10):
    # Create class dynamically
    globals()[f"TestConcrete{i}"] = type(
        f"TestConcrete{i}",
        (TestLevel3A if i % 2 == 0 else TestLevel3B,),
        {
            f"test_concrete_{j}": lambda self, j=j: None
            for j in range(10)
        }
    ) 