# Example 13 - Apptainer Extensions

Showcases the `x-apptainer` extension fields that unlock Apptainer-specific features beyond standard Docker Compose syntax. Eight services demonstrate the full range of extensions -- from GPU passthrough and fakeroot privileges to sandbox mode, overlay filesystems, security policies, and native Apptainer definition file builds.

```mermaid
graph TD
    subgraph compat["Docker-Compatible Mode"]
        Standard["standard\ndefault --compat"]
    end

    subgraph native_mode["Native Apptainer Mode"]
        Native["native\ncompat: false\nhost filesystem access"]
    end

    subgraph privileged["Elevated Privileges"]
        Installer["installer\nfakeroot: true\napt-get install"]
    end

    subgraph gpu["GPU Acceleration"]
        ML["ml-training\nnv: true\nbind_extra mounts\nwritable_tmpfs"]
        ROCM["rocm-compute\nrocm: true\nAMD GPU"]
    end

    subgraph filesystem["Filesystem Options"]
        Sandbox["dev-sandbox\nsandbox: true\noverlay image"]
    end

    subgraph security["Security Hardening"]
        Secured["secured\nseccomp profile\ncontainall\ncleanenv"]
    end

    subgraph custom["Custom Build"]
        CustomBuild["custom-build\ndef_file: ./my-image.def\nnative Apptainer build"]
    end

    XApt["x-apptainer\nextension fields"] --> compat
    XApt --> native_mode
    XApt --> privileged
    XApt --> gpu
    XApt --> filesystem
    XApt --> security
    XApt --> custom

    classDef default_svc fill:#47854b,stroke:#2d5e30,color:#fff
    classDef native_svc fill:#3498db,stroke:#2176ad,color:#fff
    classDef priv_svc fill:#e67e22,stroke:#c0651a,color:#fff
    classDef gpu_svc fill:#e74c3c,stroke:#c0392b,color:#fff
    classDef fs_svc fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef sec_svc fill:#2c3e50,stroke:#1a252f,color:#fff
    classDef build_svc fill:#d4a843,stroke:#a07d2e,color:#000
    classDef hub fill:#4a6fa5,stroke:#2c4a7c,color:#fff

    class Standard default_svc
    class Native native_svc
    class Installer priv_svc
    class ML,ROCM gpu_svc
    class Sandbox fs_svc
    class Secured sec_svc
    class CustomBuild build_svc
    class XApt hub
```

## Usage

```bash
cd examples/13-apptainer-extensions

# Run all services
apptainer-compose up

# Run a specific service to test one feature
apptainer-compose up installer
```

## What it demonstrates

- `compat: false` -- disable Docker compatibility for native Apptainer behavior
- `fakeroot: true` -- run as fake root to install packages without real privileges
- `nv: true` -- NVIDIA GPU passthrough for CUDA workloads
- `rocm: true` -- AMD ROCm GPU passthrough
- `bind_extra` -- additional host-to-container bind mounts
- `writable_tmpfs` -- writable temporary filesystem layer
- `sandbox: true` and `overlay` -- mutable sandbox containers with overlay images
- `containall`, `cleanenv`, and `security` -- isolation and seccomp hardening
- `def_file` -- build from a native Apptainer definition file instead of a Dockerfile
