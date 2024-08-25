

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