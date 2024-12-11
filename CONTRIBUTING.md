# Contributing

Every contribution is welcome!

## How to contribute

### Reporting bugs

If you found a bug, please open an [issue].

### Suggesting enhancements

If you have an idea for an enhancement, please open an [issue].

### Contributing documentation

If you want to contribute documentation, please open a pull request, and start a discussion on the PR.

### Contributing code

If you want to contribute code, please open an [issue] first to discuss what you want to do. Do not open a new [issue] to work on an existing [issue]. Then fork the repository, create a branch, and open a pull request.

## How to build

The repository is composed of multiple subfolders / modules.

### Prerequisites

- [mise-en-place](https://mise.jdx.dev/) to download tools, setup local environment (tools, environment variables) and to run the tasks
- [docker](https://docs.docker.com/get-started/) to run the containers and to execute some tests.
  To build the container locally you have to configure the container image store
  (see [Multi-platform | Docker Docs](https://docs.docker.com/build/building/multi-platform/#prerequisites)
  & [containerd image store | Docker Docs](https://docs.docker.com/engine/storage/containerd/))

### Build

```bash
cd {{subfolder}}
mise install

# to have list of tasks
mise tasks

# to run a task
mise run {{task}}

# to run the CI tasks
mise run ci
```

## How to test

```bash
cd {{subfolder}}
mise run test
```

## How to release

???

[issue]: https://github.com/cdviz-dev/issues "CDviz issues"
