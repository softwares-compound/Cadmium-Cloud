# Cadmium Cloud

Cadmium Cloud is a Rust-based web application designed to collect and manage logs from various client applications. It utilizes the Actix-web framework for handling HTTP requests and MongoDB for data storage. This project aims to provide a scalable and efficient logging solution for developers.

## Features

- **Log Collection**: Accepts structured log data from client applications.
- **Health Check Endpoint**: Provides a `/health` endpoint to verify the server's operational status.
- **MongoDB Integration**: Stores logs in a MongoDB database for efficient retrieval and analysis.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.56 or later)
- [MongoDB](https://www.mongodb.com/try/download/community) (version 4.4 or later)

## Installation

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/yourusername/cadmium-cloud.git
   cd cadmium-cloud
   ```

2. **Set Up Environment Variables**:

   Create a `.env` file in the project root directory with the following content:

   ```env
   MONGODB_URI=mongodb://localhost:27017
   MONGODB_DB=cadmium_cloud
   CADMIUM_HOST=0.0.0.0
   CADMIUM_PORT=8080
   ```

   Adjust the values as needed to match your environment.

3. **Build the Project**:

   ```bash
   cargo build --release
   ```

## Usage

1. **Run the Application**:

   ```bash
   cargo run --release
   ```

   The server will start and listen on the address specified in the `CADMIUM_HOST` and `CADMIUM_PORT` environment variables.

2. **Health Check**:

   To verify that the server is running, navigate to `http://localhost:8080/health` in your browser or use a tool like `curl`:

   ```bash
   curl http://localhost:8080/health
   ```

   A successful response will return:

   ```json
   {"status": "healthy"}
   ```

3. **Sending Logs**:

   Client applications can send logs to the `/logs` endpoint using an HTTP POST request with a JSON payload. The expected JSON structure is:

   ```json
   {
     "error": "Error message",
     "traceback": "Traceback details",
     "url": "URL where the error occurred",
     "method": "HTTP method used"
   }
   ```

   For example, using `curl`:

   ```bash
   curl -X POST http://localhost:8080/logs \
     -H "Content-Type: application/json" \
     -d '{
       "error": "Example error",
       "traceback": "Example traceback",
       "url": "http://example.com",
       "method": "GET"
     }'
   ```

## Project Structure

```plaintext
cadmium-cloud/
├── Cargo.lock
├── Cargo.toml
├── src/
│   ├── config.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── pool.rs
│   ├── handlers/
│   │   ├── log_handler.rs
│   │   └── mod.rs
│   ├── lib.rs
│   ├── logger.rs
│   ├── main.rs
│   ├── models/
│   │   ├── log.rs
│   │   └── mod.rs
│   ├── routes/
│   │   ├── health.rs
│   │   ├── logs.rs
│   │   └── mod.rs
│   └── services/
│       ├── log_service.rs
│       └── mod.rs
```

- `config.rs`: Handles configuration settings.
- `db/`: Contains database-related modules.
- `handlers/`: Defines request handlers for various endpoints.
- `logger.rs`: Sets up logging for the application.
- `main.rs`: Entry point of the application.
- `models/`: Defines data models used in the application.
- `routes/`: Configures application routes.
- `services/`: Contains business logic and services.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

Special thanks to the contributors of the Actix-web and MongoDB Rust driver projects for their excellent work. 