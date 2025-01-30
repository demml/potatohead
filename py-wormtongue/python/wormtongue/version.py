from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version("wormtongue")
except PackageNotFoundError:
    __version__ = "unknown"
