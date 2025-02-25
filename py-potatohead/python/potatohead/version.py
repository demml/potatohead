from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version("potatohead")
except PackageNotFoundError:
    __version__ = "unknown"
