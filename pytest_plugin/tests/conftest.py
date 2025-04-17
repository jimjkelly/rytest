import pytest

import rytest


class RytestCollector(pytest.Directory):
    """A directory collector that uses the Rytest collect method."""

    def collect(self):
        collector = rytest.Collector(
            path=str(self.path),
            config=self.config,
            ignore_collect=lambda path, config: self.ihook.pytest_ignore_collect(collection_path=path, config=config),
            collect_file=lambda path, parent: self.ihook.pytest_collect_file(file_path=path, parent=parent),
            parent=self,
        )

        return collector.collect()[0]


#@pytest.hookimpl
#def pytest_collect_file(parent, file_path):
#    return RytestCollector.from_parent(parent=parent, path=file_path)


@pytest.hookimpl
def pytest_collect_directory(path, parent):
    return RytestCollector.from_parent(parent=parent, path=path)
