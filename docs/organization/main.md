# Organization

## Documentation

All documentation lives in this parent folder (dystopia/docs). 

## Repository Layout

The project is a Rust workspace. All Rust crates are prefixed with "dys-" (from the working title of Dystopia).

All Rust crates are presumed to be library code (as in, not executables or binaries) unless named with a different prefix. For example, "dys-svc-\*" is used to prefix services that will be deployed and run in an environment, and "dys-tool-\*" is a development-only tool that will be run locally.
