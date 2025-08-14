use sea_orm::{prelude::*, DeleteResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QuerySelect, Set,
};
use std::sync::Arc;
use swaptun_models::{UserActiveModel, UserColumn, UserEntity, UserModel};

pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self, include_deleted: bool) -> Result<Vec<UserModel>, DbErr> {
        let mut query = UserEntity::find();

        if !include_deleted {
            query = query.filter(UserColumn::DeletedOn.is_null());
        }

        query.all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn find_by_username(&self, username: String) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Username.eq(username))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_email(&self, email: String) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: UserActiveModel) -> Result<UserModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: UserActiveModel) -> Result<UserModel, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        UserEntity::delete_by_id(id).exec(self.db.as_ref()).await
    }

    pub async fn soft_delete(&self, id: i32, now: DateTime) -> Result<Option<UserModel>, DbErr> {
        let user = self.find_by_id(id).await?;

        if let Some(user) = user {
            let mut active_model: UserActiveModel = user.into();
            active_model.deleted_on = Set(Some(now));
            active_model.updated_on = Set(now);

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn restore(&self, id: i32, now: DateTime) -> Result<Option<UserModel>, DbErr> {
        let user = self.find_by_id(id).await?;

        if let Some(user) = user {
            let mut active_model: UserActiveModel = user.into();
            active_model.deleted_on = Set(None);
            active_model.updated_on = Set(now);

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }
    pub async fn save(&self, model: UserActiveModel) -> Result<UserActiveModel, DbErr> {
        model.save(self.db.as_ref()).await
    }

    pub async fn search_users(
        &self,
        search_term: Option<String>,
        search_fields: Option<UserColumn>,
        include_deleted: bool,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<UserModel>, DbErr> {
        let mut query = UserEntity::find();

        // Apply deleted filter if needed
        if !include_deleted {
            query = query.filter(UserColumn::DeletedOn.is_null());
        }

        // Build the search condition
        if let (Some(field), Some(search_term)) = (search_fields, search_term) {
            let condition = match field {
                UserColumn::Username => {
                    Expr::cust_with_values::<&str, _, _>("username % ?", vec![search_term])
                }
                UserColumn::FirstName => {
                    Expr::cust_with_values::<&str, _, _>("first_name % ?", vec![search_term])
                }
                UserColumn::LastName => {
                    Expr::cust_with_values::<&str, _, _>("last_name % ?", vec![search_term])
                }
                UserColumn::Email => {
                    Expr::cust_with_values::<&str, _, _>("email % ?", vec![search_term])
                }
                _ => {
                    return Err(DbErr::Custom("Invalid search field".to_string()));
                }
            };
            query = query.filter(condition);
        }

        // Apply limit and offset
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query.all(self.db.as_ref()).await
    }
}
