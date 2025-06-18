use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, ColumnTrait, QueryFilter};
use swaptun_models::user_info::{self, Entity as UserInfoEntity, ActiveModel as UserInfoActiveModel, Model as UserInfo};
use super::model::UserInfoRequest;
use crate::error::AppError;
use swaptun_repositories::user_info_repository::UserInfoRepository;

pub struct UserInfoService {
    user_info_repository: UserInfoRepository,
}

impl UserInfoService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        UserInfoService {
            user_info_repository: UserInfoRepository::new(db),
        }
    }

    pub async fn save_user_info(&self, req: UserInfoRequest) -> Result<(), AppError> {
        let model = UserInfoActiveModel {
            user_id: Set(req.user_id),
            birthdate: Set(req.birthdate),
            gender: Set(req.gender),
            region: Set(req.region),
            consent: Set(req.consent),
            ..Default::default()
        };

        model.insert(self.user_info_repository.db())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    //renvoie une liste d'utilisateurs selon le genre
    pub async fn get_users_by_gender(&self, gender: &str) -> Result<Vec<UserInfo>, AppError> {
        let users = UserInfoEntity::find()
            .filter(user_info::Column::Gender.eq(gender))
            .all(self.user_info_repository.db())
            .await
            .map_err(AppError::from)?;

        Ok(users)
    }

}
