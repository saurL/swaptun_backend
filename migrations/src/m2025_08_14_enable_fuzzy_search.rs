use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Enable the pg_trgm extension for fuzzy string matching
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS pg_trgm")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to enable pg_trgm extension: {}", e)))?;

        // Create a GIN index on username for fuzzy search
        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX IF NOT EXISTS idx_users_username_gin ON tbl_users USING GIN (username gin_trgm_ops)")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create GIN index on username: {}", e)))?;

        // Create a GIN index on first_name for fuzzy search
        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX IF NOT EXISTS idx_users_first_name_gin ON tbl_users USING GIN (first_name gin_trgm_ops)")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create GIN index on first_name: {}", e)))?;

        // Create a GIN index on last_name for fuzzy search
        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX IF NOT EXISTS idx_users_last_name_gin ON tbl_users USING GIN (last_name gin_trgm_ops)")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create GIN index on last_name: {}", e)))?;

        // Create a GIN index on email for fuzzy search
        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX IF NOT EXISTS idx_users_email_gin ON tbl_users USING GIN (email gin_trgm_ops)")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create GIN index on email: {}", e)))?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the indexes
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_users_email_gin")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to drop email GIN index: {}", e)))?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_users_last_name_gin")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to drop last_name GIN index: {}", e)))?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_users_first_name_gin")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to drop first_name GIN index: {}", e)))?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_users_username_gin")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to drop username GIN index: {}", e)))?;

        // Disable the pg_trgm extension
        manager
            .get_connection()
            .execute_unprepared("DROP EXTENSION IF EXISTS pg_trgm")
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to disable pg_trgm extension: {}", e)))?;

        Ok(())
    }
}
