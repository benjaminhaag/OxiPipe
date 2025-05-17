# OxiPipe

**OxiPipe** is a lightweight, flexible, container-native pipeline system built in Rust. It supports both CI/CD workflows and general-purpose data pipelines.

## Why OxiPipe?

Existing solutions are either:

- **Heavyweight**, like Jenkins or Airflow
- **Too narrow**, like Drone or Woodpecker (limited scheduling and deduplication)
- **Too rigid**, like traditional cron setups

**OxiPipe** aims to provide:

- A modern Rust-based core
- Clean REST APIs
- Flexible pipeline logic
- Smart job scheduling
- Optional distributed execution

## Features

- **Container-based execution** (Docker or Podman)
- **Trigger system**
  - Cron-style schedules
  - Webhook/event-based triggers
  - Chained job dependencies
- **Job deduplication and debouncing**
  - Avoid wasting compute on outdated triggers
- **Multi-project / multi-user support**
  - Separate pipelines and access control
- **Distributed architecture (planned)**
  - Worker nodes to scale job execution
- **REST API**
  - Define pipelines, query jobs, trigger executions
  - Integrate webhooks from Git providers or other services

## Use Cases

- CI/CD for codebases
- Data scraping and transformation
- DevOps automation
- Lightweight job orchestration

## Project Status

This project is under active development. The MVP will focus on:
- Basic job execution
- YAML-based pipeline definitions
- Triggering via HTTP
- Logging and artifacts


## Example usage

```
docker container run --rm -e RUST_LOG=debug -v /var/run/docker.sock:/var/run/docker.sock oxipipe
```
