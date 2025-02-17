import pytest


class MyItem(pytest.Item):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
    
    def runtest(self):
        pass
    
    def reportinfo(self):
        return self.path, 0, f"custom test: {self.name}"

class MyCollector(pytest.Collector):
    """A custom collector to generate MyItem test cases."""

    def collect(self):
        return [MyItem.from_parent(self, name="custom_test_1")]


@pytest.hookimpl
def pytest_collection(session):
    session._notfound = []
    session._initial_parts = []
    session._collection_cache = {}

    root = MyCollector.from_parent(session, name="custom_root")
    session.items.extend(root.collect())  # Properly adding test items
    # Update the count of collected tests
    session.testscollected = len(session.items)
    return True