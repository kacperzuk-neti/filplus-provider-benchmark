use rabbitmq::ResultMessage;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct DataRepository {
    pool: PgPool,
}

#[derive(Debug, Serialize)]
pub struct BmsData {
    pub id: Uuid,
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
            "INSERT INTO bms_data (job_id, download, ping, head) VALUES ($1, $2, $3, $4)",
            job_id,
            self.result_to_json(result.download_result),
            self.result_to_json(result.ping_result),
            self.result_to_json(result.head_result)
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_data_by_job_id(&self, job_id: Uuid) -> Result<Vec<BmsData>, sqlx::Error> {
        let rows = sqlx::query_as!(
            BmsData,
            "SELECT id, download, ping, head FROM bms_data WHERE job_id = $1",
            job_id
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}
