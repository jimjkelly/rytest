from pytest import fixture


@fixture
def value():
    return 42


@fixture
def value_at_conftest():
    return 43
