# Example 04 - Volumes

Demonstrates data sharing between containers using named volumes and host bind mounts. A writer service produces data, a reader service consumes it read-only, and a third service maps a local directory into the container.

```mermaid
graph TD
    subgraph compose["apptainer-compose"]
        Writer["writer\n(alpine)"]
        Reader["reader\n(alpine)"]
        BindDemo["bind-mount-demo\n(alpine)"]
    end

    subgraph storage["Storage"]
        NamedVol[("shared-data\nnamed volume")]
        HostDir[("./local-files\nhost directory")]
    end

    Writer -- "read/write\n/data" --> NamedVol
    Reader -- "read-only\n/data:ro" --> NamedVol
    HostDir -- "bind mount\n/host-files" --> BindDemo

    classDef container fill:#47854b,stroke:#2d5e30,color:#fff
    classDef namedvol fill:#3498db,stroke:#2176ad,color:#fff
    classDef hostvol fill:#e67e22,stroke:#c0651a,color:#fff

    class Writer,Reader,BindDemo container
    class NamedVol namedvol
    class HostDir hostvol
```

## Usage

```bash
cd examples/04-volumes
mkdir -p local-files
echo "hello from host" > local-files/greeting.txt
apptainer-compose up
```

## What it demonstrates

- Named volumes shared between multiple services
- Read-only volume mounts (`:ro` suffix)
- Bind mounts mapping host directories into containers
- Top-level `volumes:` declaration for named volumes
