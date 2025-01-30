import pytest

@pytest.fixture
def value():
    return 42

@pytest.fixture
def yield_fixture():
    yield 42
    print("teardown")
    yield 43
    print("teardown")

def test_fixture(value, yield_fixture):
    assert value == 42
    assert yield_fixture == 42


def test_fixture_outer_scope(value, value_at_conftest):
    assert value == 42
    assert value_at_conftest == 43