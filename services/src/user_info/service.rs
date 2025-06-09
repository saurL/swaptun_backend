use sea_orm::{DbConn, EntityTrait, ActiveModelTrait, Set};
use swaptun_models::user_info::{Entity as UserInfoEntity, ActiveModel as UserInfoActiveModel};
use super::model::UserInfoRequest;
use crate::error::AppError;

pub struct UserInfoService {
    db: DbConn,
}

impl UserInfoService {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub async fn save_user_info(&self, req: UserInfoRequest) -> Result<(), AppError> {
        let model = UserInfoActiveModel {
            user_id: Set(req.user_id),
            birthdate: Set(req.birthdate),
            gender: Set(req.gender),
            region: Set(req.region),
            interests: Set(req.interests.join(",")),
            listening_minutes_per_day: Set(req.listening_minutes_per_day),
            main_devices: Set(req.main_devices.join(",")),
            consent: Set(req.consent),
            ..Default::default()
        };

        model.insert(&self.db)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}
