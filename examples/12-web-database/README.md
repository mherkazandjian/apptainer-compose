# Example 12 - Web Database

A realistic multi-tier web application with a PostgreSQL database, Redis cache, API server, background worker, and nginx reverse proxy. Services declare dependencies with health-check conditions so the API only starts after the database is confirmed healthy. Named volumes persist data across restarts.

```mermaid
graph TD
    Client["Client\n(browser / curl)"] -- "HTTP :80" --> Nginx

    subgraph compose["apptainer-compose"]
        Nginx["nginx\nnginx:alpine\n:80"]
        API["api\npython http.server\n:8000"]
        Worker["worker\nbackground jobs"]
        DB["db\npostgres:16-alpine\n:5432"]
        Redis["redis\nredis:7-alpine\n:6379"]
    end

    Nginx -- "depends_on" --> API
    API -- "depends_on\nservice_healthy" --> DB
    API -- "depends_on\nservice_started" --> Redis
    Worker -- "depends_on" --> DB
    Worker -- "depends_on" --> Redis

    DB --- PGVol[("pgdata\nvolume")]
    Redis --- RedisVol[("redis-data\nvolume")]

    HC_DB["healthcheck\npg_isready -U app"] -. "5s interval\n3 retries" .-> DB

    classDef proxy fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef app fill:#3498db,stroke:#2176ad,color:#fff
    classDef data fill:#47854b,stroke:#2d5e30,color:#fff
    classDef volume fill:#d4a843,stroke:#a07d2e,color:#000
    classDef external fill:#6b7280,stroke:#4b5563,color:#fff
    classDef health fill:#e67e22,stroke:#c0651a,color:#fff

    class Nginx proxy
    class API,Worker app
    class DB,Redis data
    class PGVol,RedisVol volume
    class Client external
    class HC_DB health
```

## Usage

```bash
cd examples/12-web-database
apptainer-compose up -d
curl http://localhost
```

## What it demonstrates

- Multi-tier architecture with database, cache, API, worker, and reverse proxy
- Health-check-based dependency ordering (`condition: service_healthy`)
- Named volumes for persistent data (`pgdata`, `redis-data`)
- Redis append-only file (AOF) persistence via a custom command
- A full dependency chain from nginx down to the data layer
