
import unittest


class SomeTest(unittest.TestCase):
    def test_something(self):
        self.assertEqual(1, 1)

    def test_something_else(self):
        assert 1 == 1

    def test_assert_failure(self):
        assert 1 == 2

    def utility(self):
        return "I return things"
