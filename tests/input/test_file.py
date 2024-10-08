import pytest

SOME_MODULE_GLOBAL = 1


def utility_function():
    return 1

def test_function_passes():
    assert utility_function() == 1

def test_function_fails():
    assert utility_function() != 1

@pytest.mark.skip
def test_function_skipped():
    assert utility_function() == 2

@pytest.mark.skip(reason="does not work")
def test_function_skipped_reason():
    assert utility_function() == 2


@pytest.mark.parametrize('a', [1, 2, 3])
def test_parameterized(a):
    assert a > 0

@pytest.mark.parametrize('a,b', [(1, 2), (3, 4)])
def test_parameterized_tuple(a, b):
    assert a < b

@pytest.mark.parametrize('c', ["a", "c"])
@pytest.mark.parametrize('a,b', [(1, 2), (3, 4)])
def test_parameterized_nested(a, b, c):
    assert a < b

@pytest.mark.parametrize('a', [x for x in range(3)])
def test_parameterized_expression(a):
    assert a > 0

@pytest.mark.parametrize('a', [round, sum, int, float])
def test_parameterized_functions(a):
    assert a > 0