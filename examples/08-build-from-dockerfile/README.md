# Example 08 - Build from Dockerfile

Build container images directly from a Dockerfile. apptainer-compose converts Dockerfiles into Apptainer definition files behind the scenes, letting you reuse existing Docker build workflows. This example includes a Flask application service built with a shorthand `build: .` and a worker service using the explicit `build.context` / `build.dockerfile` syntax.

```mermaid
graph TD
    subgraph build_context["Build Context (.)"]
        DF["Dockerfile\nFROM python:3.12-alpine\npip install flask\nCOPY app.py"]
        SRC["app.py"]
    end

    DF --> BuildApp["build: ."]
    DF --> BuildWorker["build:\n  context: .\n  dockerfile: Dockerfile"]

    subgraph compose["apptainer-compose"]
        BuildApp --> App["app service\nFlask application\n:8080"]
        BuildWorker --> Worker["worker service\necho 'Worker started'"]
    end

    Client["Client\n(browser / curl)"] -- "HTTP :8080" --> App

    classDef source fill:#6b7280,stroke:#4b5563,color:#fff
    classDef build fill:#d4a843,stroke:#a07d2e,color:#000
    classDef container fill:#47854b,stroke:#2d5e30,color:#fff
    classDef external fill:#4a6fa5,stroke:#2c4a7c,color:#fff

    class DF,SRC source
    class BuildApp,BuildWorker build
    class App,Worker container
    class Client external
```

## Usage

```bash
cd examples/08-build-from-dockerfile
apptainer-compose up
```

## What it demonstrates

- Building images from a Dockerfile (`build: .` shorthand)
- Explicit build configuration with `context` and `dockerfile` fields
- Dockerfile-to-Apptainer definition file conversion
- Port mapping and environment variables on built images
