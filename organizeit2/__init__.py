__version__ = "0.7.2"

# The extension links libfsspec from the fsspec_rs package, which puts its lib/ on the DLL path.
import fsspec_rs

from .core import *
from .types import *
