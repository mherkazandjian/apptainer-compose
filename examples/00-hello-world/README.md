# Example 00 - Hello World

The simplest possible apptainer-compose setup. A single Alpine container runs, prints a greeting, and exits. This is the starting point for verifying that your apptainer-compose installation works.

```mermaid
flowchart LR
    subgraph compose["apptainer-compose up"]
        direction LR
        A["apptainer-compose.yml"] --> B["hello service\n(alpine:latest)"]
    end
    B --> C["stdout:\nHello from apptainer-compose!"]

    classDef config fill:#4a6fa5,stroke:#2c4a7c,color:#fff
    classDef container fill:#47854b,stroke:#2d5e30,color:#fff
    classDef output fill:#d4a843,stroke:#a07d2e,color:#000

    class A config
    class B container
    class C output
```

## Usage

```bash
cd examples/00-hello-world
apptainer-compose up
```

## What it demonstrates

- Minimal `apptainer-compose.yml` structure
- Defining a single service with an image and a command
- Running a one-shot container that prints output and exits
