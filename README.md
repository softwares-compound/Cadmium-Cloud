# Cadmium Cloud

Cadmium Cloud is a Rust-based web application designed to manage organizations and applications, handle log data, and facilitate real-time communication through WebSocket connections.

## Features

- **Organization and Application Management**: Create and manage organizations and their associated applications.
- **Log Handling**: Receive and store log data with support for retry mechanisms.
- **WebSocket Communication**: Establish WebSocket connections for real-time data transmission.

## Project Structure

The project is organized as follows:

```
.
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── README.md
├── src
│   ├── db
│   │   ├── mod.rs
│   │   └── pool.rs
│   ├── handlers
│   │   ├── application_handler.rs
│   │   ├── log_handler.rs
│   │   ├── mod.rs
│   │   ├── organization_handler.rs
│   │   └── websocket_handler.rs
│   ├── lib.rs
│   ├── logger.rs
│   ├── main.rs
│   ├── models
│   │   ├── application.rs
│   │   ├── log.rs
│   │   ├── mod.rs
│   │   └── organization.rs
│   ├── routes
│   │   ├── applications.rs
│   │   ├── health.rs
│   │   ├── logs.rs
│   │   ├── mod.rs
│   │   ├── organizations.rs
│   │   └── websocket.rs
│   ├── services
│   │   ├── log_service.rs
│   │   ├── mod.rs
│   │   └── websocket_queue.rs
│   └── websocket
│       ├── connection.rs
│       ├── mod.rs
│       └── server.rs
└── test.txt
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [MongoDB](https://www.mongodb.com/try/download/community)

### Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/softwares-compound/Neocadmium-Cloud.git
   cd Neocadmium-Cloud
   ```

2. **Set up environment variables**:

   Create a `.env` file in the root directory with the following content:

   ```
   MONGODB_URI=mongodb://localhost:27017
   MONGODB_DB=cadmium_cloud_db
   ```

   Adjust the values as needed.

3. **Build and run the application**:

   ```bash
   cargo build --release
   ./target/release/cadmium-cloud
   ```

The server will start on `http://0.0.0.0:8080`.

## Usage

### API Endpoints

- **Health Check**:

  ```
  GET /health
  ```

  Response:

  ```json
  {
    "status": "healthy"
  }
  ```

- **Organizations**:

  - Create an organization:

    ```
    POST /organizations
    ```

    Request body:

    ```json
    {
      "org_name": "Your Organization Name",
      "admin_email": "admin@example.com",
      "admin_password": "yourpassword",
      "cd_id": "your_cd_id",
      "cd_secret": "your_cd_secret"
    }
    ```

    Response:

    ```json
    {
      "message": "Organization created"
    }
    ```

- **Applications**:

  - Create an application:

    ```
    POST /applications
    ```

    Request body:

    ```json
    {
      "organization_id": "organization_object_id",
      "application_name": "Your Application Name"
    }
    ```

    Response:

    ```json
    {
      "message": "Application created"
    }
    ```

- **Logs**:

  - Save a log:

    ```
    POST /logs
    ```

    Headers:

    ```
    CD-ID: your_cd_id
    CD-Secret: your_cd_secret
    Application-ID: application_object_id
    ```

    Request body:

    ```json
    {
      "error": "Error message",
      "traceback": "Traceback details",
      "url": "URL where error occurred",
      "method": "HTTP method"
    }
    ```

    Response:

    ```json
    {
      "message": "Log saved"
    }
    ```

- **WebSocket**:

  - Establish a WebSocket connection:

    ```
    GET /ws
    ```

    Headers:

    ```
    CD-ID: your_cd_id
    CD-Secret: your_cd_secret
    Application-ID: application_object_id
    ```

    Upon successful connection, the server will send real-time updates.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details. 
