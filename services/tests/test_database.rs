use std::sync::Arc;
use swaptun_services::{CreateUserRequest, UserService};
use testcontainers_modules::{
    postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

use sea_orm::{Database, DatabaseConnection};
use swaptun_migrations::{Migrator, MigratorTrait};
pub struct TestDatabase {
    pub container: Arc<ContainerAsync<postgres::Postgres>>,
    db: Arc<DatabaseConnection>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let (db, container) = setup_db().await;
        TestDatabase { container, db }
    }

    pub fn get_db(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }

    pub async fn drop(&self) {
        self.container.stop().await.unwrap();
    }
}

pub async fn setup_container() -> Arc<ContainerAsync<postgres::Postgres>> {
    let container: testcontainers_modules::testcontainers::ContainerAsync<postgres::Postgres> =
        postgres::Postgres::default().start().await.unwrap();

    Arc::new(container)
}

pub async fn setup_db() -> (
    Arc<DatabaseConnection>,
    Arc<ContainerAsync<postgres::Postgres>>,
) {
    let container = setup_container().await;
    let host_ip = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    let connection_string = format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host_ip, host_port
    );
    let db = Database::connect(&connection_string)
        .await
        .expect("Failed to connect to test Postgres DB");

    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let db = Arc::new(db);

    let user_service = UserService::new(db.clone());
    let create_user_request = CreateUserRequest {
        username: "unique_user".to_string(),
        password: "hashed_passwor12D!".to_string(),
        first_name: "first_name".to_string(),
        last_name: "last_name".to_string(),
        email: "unique_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await;

    (db, container)
}
