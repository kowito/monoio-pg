# Contributing to monoio-pg

Thank you for your interest in contributing to `monoio-pg`! This project aims to be the fastest and most efficient PostgreSQL driver for the `monoio` runtime.

## How to Contribute

1.  **Fork the repository** and create your branch from `main`.
2.  **Ensure the code builds** and passes all tests:
    ```bash
    cargo check
    cargo test --test integration_test -- --nocapture
    ```
3.  **Run benchmarks** if you are proposing performance improvements:
    ```bash
    cargo bench --bench benchmark
    ```
4.  **Keep it lean**: We prioritize performance and minimal dependencies. Avoid adding new dependencies unless absolutely necessary.
5.  **Follow the style**: Use standard Rust formatting (`cargo fmt`).

## Development Environment

You'll need a running PostgreSQL instance for integration tests. The tests expect:
- Host: `127.0.0.1:5432`
- User: `monoio`
- Password: `monoio`
- Database: `postgres`

## Bug Reports & Feature Requests

Feel free to open an issue for any bugs or suggestions. Please provide as much context as possible, including your OS, Rust version, and a reproduction case if applicable.

## License

By contributing, you agree that your contributions will be licensed under the project's [WTFPL](LICENSE) license.
