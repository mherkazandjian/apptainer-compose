# Example 11 - Resource Limits

Apply CPU and memory constraints to containers using the `deploy.resources` section. apptainer-compose translates these into Apptainer cgroup flags (`--cpus`, `--memory`, `--pids-limit`). Three services demonstrate different resource profiles -- from a CPU-intensive workload to a minimal background task.

```mermaid
graph TD
    subgraph compose["apptainer-compose"]
        direction TB
        CPU["cpu-bound\nbusy loop"]
        MEM["memory-bound\n64M urandom read"]
        LIGHT["lightweight\nsleep infinity"]
    end

    subgraph cpu_res["cpu-bound resources"]
        direction LR
        CPUL["Limits\n0.5 CPU | 256M RAM | 100 PIDs"]
        CPUR["Reservations\n0.25 CPU | 128M RAM"]
    end

    subgraph mem_res["memory-bound resources"]
        MEML["Limits\n1.0 CPU | 512M RAM"]
    end

    subgraph light_res["lightweight resources"]
        LIGHTL["Limits\n0.1 CPU | 64M RAM"]
    end

    CPU --- cpu_res
    MEM --- mem_res
    LIGHT --- light_res

    classDef heavy fill:#e74c3c,stroke:#c0392b,color:#fff
    classDef medium fill:#e67e22,stroke:#c0651a,color:#fff
    classDef light fill:#47854b,stroke:#2d5e30,color:#fff
    classDef limit fill:#4a6fa5,stroke:#2c4a7c,color:#fff
    classDef reserve fill:#6b7280,stroke:#4b5563,color:#fff

    class CPU heavy
    class MEM medium
    class LIGHT light
    class CPUL,MEML,LIGHTL limit
    class CPUR reserve
```

## Usage

```bash
cd examples/11-resource-limits
apptainer-compose up -d
```

## What it demonstrates

- Setting CPU limits (`cpus`) and memory limits (`memory`) via `deploy.resources.limits`
- Setting resource reservations via `deploy.resources.reservations`
- PID limits (`pids`) to restrict the number of processes
- Translation of Docker Compose resource syntax to Apptainer cgroup flags
