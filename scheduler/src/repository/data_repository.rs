use rabbitmq::ResultMessage;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct DataRepository {
    pool: PgPool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BmsData {
    pub id: Uuid,
    pub worker_name: Option<String>,
    pub download: serde_json::Value,
    pub ping: serde_json::Value,
    pub head: serde_json::Value,
}

impl DataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn result_to_json<T: serde::Serialize, E: serde::Serialize>(
        &self,
        option: Result<T, E>,
    ) -> serde_json::Value {
        match option {
            Ok(value) => serde_json::to_value(&value).unwrap_or_default(),
            Err(error) => serde_json::to_value(&error).unwrap_or_default(),
        }
    }

    pub async fn save_data(&self, job_id: Uuid, result: ResultMessage) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO worker_data (
                id, 
                job_id, 
                worker_name, 
                is_success, 
                download, 
                ping, 
                head
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            result.run_id,
            job_id,
            result.worker_name,
            result.is_success,
            self.result_to_json(result.download_result),
            self.result_to_json(result.ping_result),
            self.result_to_json(result.head_result)
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
