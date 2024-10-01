use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::data_repository::BmsData;

#[derive(Clone)]
pub struct JobRepository {
    pool: PgPool,
}

#[derive(Serialize, Debug)]
pub struct JobWithData {
    pub id: Uuid,
    pub url: Option<String>,
    pub routing_key: Option<String>,
    pub details: Option<serde_json::Value>,
    pub data: Vec<BmsData>,
}

impl JobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_job(
        &self,
        job_id: Uuid,
        url: String,
        routing_key: &String,
        details: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO jobs (id, url, routing_key, details)
            VALUES ($1, $2, $3, $4)
            "#,
            job_id,
            url,
            routing_key,
            details,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_job_by_id_with_data(&self, job_id: Uuid) -> Result<JobWithData, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                jobs.id as id, 
                jobs.url, 
                jobs.routing_key, 
                jobs.details, 
                bms_data.id as bms_id, 
                bms_data.job_id as bms_job_id, 
                bms_data.worker_name,
                bms_data.download,
                bms_data.ping,
                bms_data.head
            FROM jobs 
            LEFT JOIN bms_data ON jobs.id = bms_data.job_id
            WHERE jobs.id = $1
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Err(sqlx::Error::RowNotFound);
        }

        let mut job = JobWithData {
            id: rows[0].id,
            url: rows[0].url.clone(),
            routing_key: rows[0].routing_key.clone(),
            details: rows[0].details.clone(),
            data: Vec::new(),
        };

        let no_data = json!({"error": "no data"});

        // Collect bms_data rows
        for row in rows {
            let bms = BmsData {
                id: row.bms_id,
                worker_name: row.worker_name.clone(),
                download: row.download.unwrap_or(no_data.clone()),
                ping: row.ping.unwrap_or(no_data.clone()),
                head: row.head.unwrap_or(no_data.clone()),
            };
            job.data.push(bms);
        }

        Ok(job)
    }
}
