# organizeit2

Engage with Zorp!

[![Build Status](https://github.com/1kbgz/organizeit2/actions/workflows/build.yaml/badge.svg?branch=main&event=push)](https://github.com/1kbgz/organizeit2/actions/workflows/build.yaml)
[![codecov](https://codecov.io/gh/1kbgz/organizeit2/branch/main/graph/badge.svg)](https://codecov.io/gh/1kbgz/organizeit2)
[![License](https://img.shields.io/github/license/1kbgz/organizeit2)](https://github.com/1kbgz/organizeit2)
[![PyPI](https://img.shields.io/pypi/v/organizeit2.svg)](https://pypi.python.org/pypi/organizeit2)

> This morning at dawn, you will take a new form - that of a fleshless, chattering skeleton when Zorp the Surveyor arrives and burns your flesh off with his volcano mouth ~Lou Prozotovich

`OrganizeIt2` is a python library for managing large numbers of files and directories. It is type- and configuration-driven with [pydantic](https://docs.pydantic.dev/latest/).

The name is because `organizeit` was [taken on pypi](https://pypi.org/project/organizeit/), and is thus a reference to the [joke from Parks and Rec](https://parksandrecreation.fandom.com/wiki/The_Reasonabilists).

## Overview

<<<<<<< before updating
`OrganizeIt2` has the following models and types:

- `FileSystem`: `pydantic` wrapper of an `fsspec` `AbstractFileSystem`
- `Path`: wrapper of an `fsspec` path
- `FilePath`: specialization of a `Path` for files
- `DirectoryPath`: specialization of a `Path` for directories
- `OrganizeIt`: Top-level `pydantic` model representiing an `fsspec` directory
- `Directory`: `pydantic` model representing an `fsspec` directory
- `File`: `pydantic` model representing an `fsspec` file

=======
>>>>>>> after updating
> [!NOTE]
> This library was generated using [copier](https://copier.readthedocs.io/en/stable/) from the [Base Python Project Template repository](https://github.com/python-project-templates/base).
