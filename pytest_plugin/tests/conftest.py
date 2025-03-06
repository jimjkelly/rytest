import pytest



class MyItem(pytest.Item):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
    
    
    def reportinfo(self):
        return self.path, 0, f"custom test: {self.name}"

class MyCollector(pytest.Collector):
    """A custom collector to generate MyItem test cases."""

    def collect(self):
        return [MyItem.from_parent(self, name="custom_test_1")]

@pytest.hookimpl
def pytest_runtestloop(session) -> bool:
    if session.testsfailed and not session.config.option.continue_on_collection_errors:
        raise session.Interrupted(
            f"{session.testsfailed} error{'s' if session.testsfailed != 1 else ''} during collection"
        )

    if session.config.option.collectonly:
        return True
    
    for i, item in enumerate(session.items):
        nextitem = session.items[i + 1] if i + 1 < len(session.items) else None
        item.config.hook.pytest_runtest_protocol(item=item, nextitem=nextitem)
        if session.shouldfail:
            raise session.Failed(session.shouldfail)
        if session.shouldstop:
            raise session.Interrupted(session.shouldstop)
    return True


@pytest.hookimpl
def pytest_collection(session):

    hook = session.config.hook

    root = MyCollector.from_parent(session, name="custom_root")
    session.items.extend(root.collect())
    session.items.extend(root.collect())

    session.testscollected = len(session.items)
    hook.pytest_collection_modifyitems(
                session=session, config=session.config, items=session.items
    )

    session._notfound = []
    session._initial_parts = []
    session._collection_cache = {}
    hook.pytest_collection_finish(session=session)
    
    return session.items