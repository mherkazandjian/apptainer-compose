# Example 09 - Multi-File Override

Separate configuration into a base compose file and an override file. The base file defines production defaults, while the override file layers development-specific changes on top -- different ports, extra environment variables, and debug settings. This pattern keeps production config clean and lets developers customize behavior without editing the shared file.

```mermaid
graph LR
    subgraph base["apptainer-compose.yml (Base)"]
        direction TB
        WebBase["web\nnginx:alpine\n:80\nENV=production"]
        ApiBase["api\npython http.server\n:5000"]
    end

    subgraph override["apptainer-compose.override.yml"]
        direction TB
        WebOver["web\n:8080\nENV=development\nDEBUG=true"]
        ApiOver["api\n:5001\nLOG_LEVEL=debug"]
    end

    subgraph merged["Merged Result"]
        direction TB
        WebFinal["web\nnginx:alpine\n:8080\nENV=development\nDEBUG=true"]
        ApiFinal["api\npython http.server\n:5001\nLOG_LEVEL=debug"]
    end

    WebBase -- "overridden by" --> WebOver
    ApiBase -- "overridden by" --> ApiOver
    WebOver -- "produces" --> WebFinal
    ApiOver -- "produces" --> ApiFinal

    classDef prod fill:#47854b,stroke:#2d5e30,color:#fff
    classDef dev fill:#e67e22,stroke:#c0651a,color:#fff
    classDef merged fill:#3498db,stroke:#2176ad,color:#fff

    class WebBase,ApiBase prod
    class WebOver,ApiOver dev
    class WebFinal,ApiFinal merged
```

## Usage

```bash
cd examples/09-multi-file-override

# Run with both files (base + override)
apptainer-compose -f apptainer-compose.yml -f apptainer-compose.override.yml up

# Run base only (production defaults)
apptainer-compose -f apptainer-compose.yml up
```

## What it demonstrates

- Splitting configuration across multiple compose files
- Override files that modify ports, environment variables, and other settings
- The `-f` flag to specify which compose files to merge
- A clean pattern for separating production and development configuration
