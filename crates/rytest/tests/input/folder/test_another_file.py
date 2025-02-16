from .utils import util

def test_another_function():
    pass


def accepts(*types):
    def check_accepts(f):
        assert len(types) == f.__code__.co_argcount
        def new_f(*args, **kwds):
            for (a, t) in zip(args, types):
                assert isinstance(a, t), \
                       "arg %r does not match %s" % (a,t)
            return f(*args, **kwds)
        new_f.__name__ = f.__name__
        return new_f
    return check_accepts


@accepts(int, (int, float))
def test_function_with_decorator(arg1, arg2):
    try:
        util()
    except AssertionError:
        pass
