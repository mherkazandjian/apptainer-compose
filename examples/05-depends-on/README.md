# Example 05 - Depends On

Four services with an explicit dependency chain that controls startup order. The database starts first, then the migration runs, then the API, and finally the frontend -- each waiting for its dependencies before launching.

```mermaid
graph BT
    DB["db\n(postgres)"]
    Migrate["migrate\n(schema setup)"]
    API["api\n(backend)"]
    Frontend["frontend\n(web UI)"]

    Migrate -- "depends_on" --> DB
    API -- "depends_on" --> DB
    API -- "depends_on" --> Migrate
    Frontend -- "depends_on" --> API

    StartOrder["Startup order:\n1. db\n2. migrate\n3. api\n4. frontend"]

    classDef database fill:#3498db,stroke:#2176ad,color:#fff
    classDef migration fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef backend fill:#47854b,stroke:#2d5e30,color:#fff
    classDef front fill:#e67e22,stroke:#c0651a,color:#fff
    classDef note fill:#f0f0f0,stroke:#999,color:#333

    class DB database
    class Migrate migration
    class API backend
    class Frontend front
    class StartOrder note
```

## Usage

```bash
cd examples/05-depends-on
apptainer-compose up
```

## What it demonstrates

- Service startup ordering with `depends_on:`
- Dependency chains across multiple levels
- Ensuring a database is running before migrations execute
- Ensuring migrations complete before the API starts
- Sequential and fan-in dependency patterns
