# Example 02 - Multi-Service

Three independent services running side by side: an HTTP API, a background worker, and a Redis cache. This demonstrates orchestrating multiple containers from a single compose file.

```mermaid
graph TD
    subgraph compose["apptainer-compose"]
        API["api\npython http.server\n:5000"]
        Worker["worker\nalpine tick loop\n(every 10s)"]
        Cache["cache\nredis 7\n:6379"]
    end

    ClientHTTP["HTTP Client"] -- "port 5000" --> API
    ClientRedis["Redis Client"] -- "port 6379" --> Cache
    Worker -- "stdout:\nworker tick" --> Logs["Container Logs"]

    classDef api fill:#3498db,stroke:#2176ad,color:#fff
    classDef worker fill:#e67e22,stroke:#c0651a,color:#fff
    classDef cache fill:#e74c3c,stroke:#c0392b,color:#fff
    classDef external fill:#6b7280,stroke:#4b5563,color:#fff
    classDef output fill:#d4a843,stroke:#a07d2e,color:#000

    class API api
    class Worker worker
    class Cache cache
    class ClientHTTP,ClientRedis external
    class Logs output
```

## Usage

```bash
cd examples/02-multi-service
apptainer-compose up -d
curl http://localhost:5000
redis-cli -p 6379 ping
```

## What it demonstrates

- Running multiple independent services from one compose file
- Different image sources (Python, Alpine, Redis)
- Multiple port mappings across services
- Long-running background processes (worker loop)
