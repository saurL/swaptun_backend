use crate::auth::{Claims, generate_token, verify_password};
use crate::validators::user_validators::process_validation_errors;
use crate::{
    auth::{hash_password, validate_token},
    error::AppError,
};
use chrono::{Local, NaiveDateTime};
use log::{info, warn};
use sea_orm::{ActiveValue::Set, DatabaseConnection, DbErr, DeleteResult};
use std::sync::Arc;
use swaptun_models::{UserActiveModel, UserModel};
use swaptun_repositories::UserRepository;

use crate::dto::{
    CreateUserRequest, GetUsersParams, LoginEmailRequest, LoginRequest, LoginResponse,
    UpdateUserRequest, VerifyTokenRequest, VerifyTokenResponse,
};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        UserService {
            user_repository: UserRepository::new(db),
        }
    }

    pub async fn find_all(&self, include_deleted: bool) -> Result<Vec<UserModel>, DbErr> {
        self.user_repository.find_all(include_deleted).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<UserModel>, DbErr> {
        self.user_repository.find_by_id(id).await
    }

    pub async fn find_by_username(&self, username: String) -> Result<Option<UserModel>, DbErr> {
        self.user_repository.find_by_username(username).await
    }

    pub async fn find_by_email(&self, email: String) -> Result<Option<UserModel>, DbErr> {
        self.user_repository.find_by_email(email).await
    }

    async fn create(&self, model: UserActiveModel) -> Result<UserModel, DbErr> {
        self.user_repository.create(model).await
    }

    pub async fn update(&self, model: UserActiveModel) -> Result<UserModel, DbErr> {
        self.user_repository.update(model).await
    }
    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        self.user_repository.delete(id).await
    }

    pub async fn soft_delete(
        &self,
        id: i32,
        now: NaiveDateTime,
    ) -> Result<Option<UserModel>, DbErr> {
        self.user_repository.soft_delete(id, now).await
    }

    pub async fn restore(&self, id: i32, now: NaiveDateTime) -> Result<Option<UserModel>, DbErr> {
        self.user_repository.restore(id, now).await
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserModel, AppError> {
        info!(
            "Attempting to create user with username: {}",
            request.username
        );

        process_validation_errors(&request)?;
        if let Some(_) = self.find_by_username(request.username.clone()).await? {
            return Err(AppError::Validation(format!(
                "Username {} already exists",
                request.username
            )));
        }
        if let Some(_) = self.find_by_email(request.email.clone()).await? {
            return Err(AppError::Validation(format!(
                "Email {} already exists",
                request.email
            )));
        }

        let now = Local::now().naive_local();
        let hashed_password = hash_password(&request.password)?;
        let user_model = UserActiveModel {
            username: Set(request.username.clone()),
            password: Set(hashed_password),
            first_name: Set(request.first_name.clone()),
            last_name: Set(request.last_name.clone()),
            email: Set(request.email.clone()),
            role: Set("user".to_string()),
            created_on: Set(now),
            updated_on: Set(now),
            ..Default::default()
        };

        let user = self.create(user_model).await?;

        info!("User created with ID: {}", user.id);
        Ok(user)
    }

    pub async fn get_users(&self, request: GetUsersParams) -> Result<Vec<UserModel>, DbErr> {
        let include_deleted = request.include_deleted.unwrap_or(false);
        let users = self.find_all(include_deleted).await?;
        Ok(users)
    }

    pub async fn get_user(&self, id: i32) -> Result<Option<UserModel>, DbErr> {
        let user = self.find_by_id(id).await?;
        Ok(user)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
        process_validation_errors(&request)?;

        let user = match self.find_by_username(request.username.clone()).await? {
            Some(user) => user,
            None => return Err(AppError::Unauthorized("Invalid credentials".into())),
        };

        let is_valid = verify_password(&request.password, &user.password)?;
        if !is_valid {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        if user.deleted_on.is_some() {
            return Err(AppError::Unauthorized("Account is disabled".into()));
        }

        let token = generate_token(&user)?;

        Ok(LoginResponse {
            token,
            user_id: user.id,
            username: user.username,
            role: user.role,
        })
    }

    pub async fn update_user(
        &self,
        request: UpdateUserRequest,
        id: i32,
    ) -> Result<UserModel, AppError> {
        process_validation_errors(&request)?;

        info!("Attempting to update user with ID: {}", id);

        if let Some(ref username) = request.username {
            if username.trim().is_empty() {
                return Err(AppError::Validation("Username cannot be empty".into()));
            }

            if let Some(existing_user) = self.find_by_username(username.clone()).await? {
                if existing_user.id != id {
                    return Err(AppError::Validation(format!(
                        "Username {} already exists",
                        username
                    )));
                }
            }
        }

        if let Some(ref email) = request.email {
            if email.trim().is_empty() {
                return Err(AppError::Validation("Email cannot be empty".into()));
            }

            if let Some(existing_user) = self.find_by_email(email.clone()).await? {
                if existing_user.id != id {
                    return Err(AppError::Validation(format!(
                        "Email {} already exists",
                        email
                    )));
                }
            }
        }

        let user = self.find_by_id(id).await?;

        match user {
            Some(user) => {
                let mut active_model: UserActiveModel = user.into();

                if let Some(username) = &request.username {
                    active_model.username = Set(username.clone());
                }
                if let Some(first_name) = &request.first_name {
                    active_model.first_name = Set(first_name.clone());
                }
                if let Some(last_name) = &request.last_name {
                    active_model.last_name = Set(last_name.clone());
                }
                if let Some(email) = &request.email {
                    active_model.email = Set(email.clone());
                }

                active_model.role = Set("user".to_string());

                active_model.updated_on = Set(Local::now().naive_local());

                let updated_user = self.update(active_model).await?;

                info!("User with ID {} updated", id);
                Ok(updated_user)
            }
            None => Err(AppError::NotFound(format!("User with ID {} not found", id))),
        }
    }

    pub async fn delete_user_physical(&self, user_id: i32) -> Result<(), AppError> {
        info!("Attempting to physically delete user with ID: {}", user_id);

        let user = self.find_by_id(user_id).await?;
        if user.is_none() {
            return Err(AppError::NotFound(format!(
                "User with ID {} not found",
                user_id
            )));
        }

        let delete_result = self.delete(user_id).await?;

        if delete_result.rows_affected > 0 {
            info!("User with ID {} successfully deleted physically", user_id);
            Ok(())
        } else {
            warn!("User with ID {} was not deleted (0 rows affected)", user_id);
            Err(AppError::InternalServerError)
        }
    }

    pub async fn delete_user_logical(&self, user_id: i32) -> Result<(), AppError> {
        info!("Attempting to logically delete user with ID: {}", user_id);

        let user = self.find_by_id(user_id).await?;

        match user {
            Some(user) => {
                if user.deleted_on.is_some() {
                    warn!("User with ID {} is already logically deleted", user_id);
                    return Err(AppError::Validation(format!(
                        "User with ID {} is already marked as deleted",
                        user_id
                    )));
                }

                let now = Local::now().naive_local();
                let result = self.soft_delete(user_id, now).await?;

                if result.is_some() {
                    info!("User with ID {} successfully marked as deleted", user_id);
                    Ok(())
                } else {
                    Err(AppError::InternalServerError)
                }
            }
            None => Err(AppError::NotFound(format!(
                "User with ID {} not found",
                user_id
            ))),
        }
    }

    pub async fn restore_user(&self, user_id: i32) -> Result<(), AppError> {
        info!(
            "Attempting to restore logically deleted user with ID: {}",
            user_id
        );

        let user = self.find_by_id(user_id).await?;

        match user {
            Some(user) => {
                if user.deleted_on.is_none() {
                    warn!("User with ID {} is not deleted, cannot restore", user_id);
                    return Err(AppError::Validation(format!(
                        "User with ID {} is not marked as deleted",
                        user_id
                    )));
                }

                let now = Local::now().naive_local();
                let result = self.restore(user_id, now).await?;

                if result.is_some() {
                    info!("User with ID {} successfully restored", user_id);
                    Ok(())
                } else {
                    Err(AppError::InternalServerError)
                }
            }
            None => Err(AppError::NotFound(format!(
                "User with ID {} not found",
                user_id
            ))),
        }
    }

    pub async fn login_with_email(
        &self,
        request: LoginEmailRequest,
    ) -> Result<LoginResponse, AppError> {
        process_validation_errors(&request)?;

        let user = match self.find_by_email(request.email.clone()).await? {
            Some(user) => user,
            None => return Err(AppError::Unauthorized("Invalid credentials".into())),
        };

        let is_valid = verify_password(&request.password, &user.password)?;
        if !is_valid {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        if user.deleted_on.is_some() {
            return Err(AppError::Unauthorized("Account is disabled".into()));
        }

        let token = generate_token(&user)?;

        Ok(LoginResponse {
            token,
            user_id: user.id,
            username: user.username,
            role: user.role,
        })
    }

    pub async fn verify_token(
        &self,
        request: VerifyTokenRequest,
    ) -> Result<VerifyTokenResponse, AppError> {
        match validate_token(&request.token) {
            Ok(data) => {
                let user = self.find_by_id(data.claims.user_id).await?;
                match user {
                    Some(_) => Ok(VerifyTokenResponse { valid: true }),
                    None => Ok(VerifyTokenResponse { valid: false }),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_user_from_claims(&self, claims: Claims) -> Result<UserModel, AppError> {
        let user = self.find_by_id(claims.user_id).await?;
        match user {
            Some(user) => Ok(user),
            None => Err(AppError::NotFound(format!(
                "User with ID {} not found",
                claims.user_id
            ))),
        }
    }
}
