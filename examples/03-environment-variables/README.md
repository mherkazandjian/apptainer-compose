# Example 03 - Environment Variables

Three services showcasing the different ways to inject configuration into containers: inline key-value pairs, an external `.env` file, and variable interpolation with default values.

```mermaid
graph TD
    subgraph sources["Configuration Sources"]
        Inline["Inline environment:\nDB_HOST=localhost\nDB_PORT=5432\nAPP_ENV=production"]
        EnvFile[".env file\n(key=value pairs)"]
        Defaults["Shell / Defaults\nGREETING with fallback"]
    end

    subgraph compose["apptainer-compose"]
        App["app service\n(prints inline vars)"]
        AppFile["app-from-file service\n(prints env | sort)"]
        AppDefaults["app-with-defaults service\n(prints GREETING)"]
    end

    Inline -- "environment:" --> App
    EnvFile -- "env_file:" --> AppFile
    Defaults -- "${GREETING:-Hello World}" --> AppDefaults

    App --> Out1["DB_HOST=localhost\nDB_PORT=5432\nAPP_ENV=production"]
    AppFile --> Out2["All variables\nfrom .env sorted"]
    AppDefaults --> Out3["GREETING=Hello World\n(or overridden value)"]

    classDef source fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef container fill:#47854b,stroke:#2d5e30,color:#fff
    classDef output fill:#d4a843,stroke:#a07d2e,color:#000

    class Inline,EnvFile,Defaults source
    class App,AppFile,AppDefaults container
    class Out1,Out2,Out3 output
```

## Usage

```bash
cd examples/03-environment-variables

# Run with defaults
apptainer-compose up

# Override an interpolated variable
GREETING="Howdy" apptainer-compose up
```

## What it demonstrates

- Inline `environment:` key-value definitions
- Loading variables from an external file with `env_file:`
- Variable interpolation using `${VAR:-default}` syntax
- How to override interpolated variables from the host shell
