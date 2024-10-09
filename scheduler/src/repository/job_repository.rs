use serde::Serialize;
use sqlx::{
    prelude::{FromRow, Type},
    PgPool,
};
use uuid::Uuid;

use super::data_repository::BmsData;

#[derive(Clone)]
pub struct JobRepository {
    pool: PgPool,
}

#[derive(Serialize, Debug, FromRow, Type)]
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
        let job = sqlx::query_as!(
            JobWithData,
            r#"
            SELECT
                jobs.id,
                jobs.url,
                jobs.routing_key,
                jobs.details,
                COALESCE(
                    ARRAY_AGG(
                        JSON_BUILD_OBJECT(
                            'id', d.id,
                            'worker_name', d.worker_name,
                            'download', d.download,
                            'ping', d.ping,
                            'head', d.head
                        )
                    ) FILTER (WHERE d.id IS NOT NULL),
                    ARRAY[]::json[]
                ) AS "data!: Vec<BmsData>"
            FROM jobs
            LEFT JOIN bms_data as d ON jobs.id = d.job_id
            WHERE jobs.id = $1
            GROUP BY jobs.id
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(job)
    }
}
