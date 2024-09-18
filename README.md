# Bandwith Measurement System (BMS)


## Infrastucture
- Scheduler (Rust)
- Worker (Rust)
- PostgresSQL
- RabbitMQ

## Environment variables

Example env file: [.env.example](./.env.example)

Global ENV:
- `DATABASE_URL`: URI to the postgres database
- `RABBITMQ_HOST`: Hostname of the RabbitMQ server
- `LOG_LEVEL`: Log level of the application (debug, info, warn, error)

Worker ENV:
- `WORKER_NAME`: Unique identifier of the worker, user to bind to the queue

## RabbitMQ Communication

### Job Exchange

![Job Exchange](./docs/bms_queue_job_1.drawio.png)

### Result Exchange

![Result Exchange](./docs/bms_queue_results_1.drawio.png)