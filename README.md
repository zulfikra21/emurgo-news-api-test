
# Actix Scaffolding

## Installation

### this API was build with my existing actix scaffolding

## Prerequisites
-   Rust 1.75 or latest version  
-   make sure you've installed openssl version 3 if you want to run using Docker

## How to Run
- Docker compose : Run the app using the command ``./start.sh``  
- Linux CLI directly : 
    - run ```docker compose up -d redis```
    - change REDIS_URL inside ```.env``` file from ``redis://redis:6378`` to ```redis://localhost:6378``` then run command ```cargo run```

