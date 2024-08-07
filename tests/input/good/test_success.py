

def test_success():
    assert True


def test_more_success():
    assert True

@pytest.fixture
def test_fixture():
    return "fixtures starting with test_ should be ignored during test collection"