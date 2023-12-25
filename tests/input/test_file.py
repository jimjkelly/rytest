

SOME_MODULE_GLOBAL = 1


def utility_function():
    return 1

def test_function_passes():
    assert utility_function() == 1

def test_function_fails():
    assert utility_function() != 1
