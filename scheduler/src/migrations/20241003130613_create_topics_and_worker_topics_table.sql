-- Create the topics table
CREATE TABLE IF NOT EXISTS topics (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

-- Create the worker_topics join table
CREATE TABLE  IF NOT EXISTS worker_topics (
    worker_name VARCHAR NOT NULL,
    topic_id INT NOT NULL,
    PRIMARY KEY (worker_name, topic_id),
    FOREIGN KEY (worker_name) REFERENCES workers(worker_name) ON DELETE CASCADE,
    FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
);
