# Example 06 - Healthcheck

A PostgreSQL database with a health check and an application that waits until the database reports healthy before starting. This goes beyond simple `depends_on` ordering by verifying the service is actually ready to accept connections.

```mermaid
graph LR
    subgraph compose["apptainer-compose"]
        DB["db\n(postgres:16-alpine)"]
        HC{"healthcheck\npg_isready -U postgres\ninterval: 5s\ntimeout: 3s\nretries: 5\nstart_period: 10s"}
        App["app\n(application)"]
    end

    DB --- HC
    HC -- "condition:\nservice_healthy" --> App

    Starting["starting"] --> Healthy["healthy"]
    Healthy --> AppStarts["app starts"]
    Starting -. "retries exhausted" .-> Unhealthy["unhealthy"]

    subgraph lifecycle["Health Lifecycle"]
        Starting
        Healthy
        Unhealthy
    end

    classDef database fill:#3498db,stroke:#2176ad,color:#fff
    classDef check fill:#f39c12,stroke:#d68910,color:#000
    classDef app fill:#47854b,stroke:#2d5e30,color:#fff
    classDef ok fill:#27ae60,stroke:#1e8449,color:#fff
    classDef fail fill:#e74c3c,stroke:#c0392b,color:#fff
    classDef pending fill:#95a5a6,stroke:#7f8c8d,color:#fff

    class DB database
    class HC check
    class App app
    class Healthy,AppStarts ok
    class Unhealthy fail
    class Starting pending
```

## Usage

```bash
cd examples/06-healthcheck
apptainer-compose up
```

## What it demonstrates

- Defining a `healthcheck:` with test command, interval, timeout, retries, and start period
- Using `depends_on:` with `condition: service_healthy`
- Difference between startup ordering (`depends_on`) and readiness verification (healthcheck)
- Reliable database-backed application startup
