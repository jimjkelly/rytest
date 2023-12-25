import pytest


@pytest.mark.parametrize("a,b,expected", [(3, 5, 8), (2, 4, 6)])
def test_parameterized(a, b, expected):
    assert a + b == expected


def test_false():
    assert True
