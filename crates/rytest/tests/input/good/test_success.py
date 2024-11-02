import pytest
from tests.input.good.utils import assert_true

def test_success():
    assert assert_true(True)


def test_more_success():
    assert True


@pytest.fixture
def test_fixture():
    return "fixtures starting with test_ should be ignored during test collection"

def test_using_fixture(test_fixture):
    assert test_fixture == "fixtures starting with test_ should be ignored during test collection"