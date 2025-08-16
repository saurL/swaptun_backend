use std::sync::Arc;
use swaptun_models::friendship::{
    ActiveModel as FriendshipActiveModel, Column as FriendshipColumn, Entity as FriendshipEntity,
    Model as FriendshipModel,
};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    QueryFilter, TransactionTrait,
};

pub struct FriendshipRepository {
    db: Arc<DatabaseConnection>,
}

impl FriendshipRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<FriendshipModel>, DbErr> {
        FriendshipEntity::find_by_id(id).one(&*self.db).await
    }

    pub async fn find_friendship(
        &self,
        user_id: i32,
        friend_id: i32,
    ) -> Result<Option<FriendshipModel>, DbErr> {
        FriendshipEntity::find()
            .filter(
                FriendshipColumn::UserId
                    .eq(user_id)
                    .and(FriendshipColumn::FriendId.eq(friend_id)),
            )
            .one(&*self.db)
            .await
    }

    pub async fn find_friendship_bidirectional(
        &self,
        user_id: i32,
        friend_id: i32,
    ) -> Result<Option<FriendshipModel>, DbErr> {
        FriendshipEntity::find()
            .filter(
                (FriendshipColumn::UserId
                    .eq(user_id)
                    .and(FriendshipColumn::FriendId.eq(friend_id)))
                .or(FriendshipColumn::UserId
                    .eq(friend_id)
                    .and(FriendshipColumn::FriendId.eq(user_id))),
            )
            .one(&*self.db)
            .await
    }

    pub async fn create(&self, model: FriendshipActiveModel) -> Result<FriendshipModel, DbErr> {
        model.insert(&*self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        FriendshipEntity::delete_by_id(id).exec(&*self.db).await
    }

    pub async fn delete_friendship(
        &self,
        user_id: i32,
        friend_id: i32,
    ) -> Result<DeleteResult, DbErr> {
        FriendshipEntity::delete_many()
            .filter(
                (FriendshipColumn::UserId
                    .eq(user_id)
                    .and(FriendshipColumn::FriendId.eq(friend_id)))
                .or(FriendshipColumn::UserId
                    .eq(friend_id)
                    .and(FriendshipColumn::FriendId.eq(user_id))),
            )
            .exec(&*self.db)
            .await
    }

    pub async fn create_mutual_friendship(
        &self,
        user_id: i32,
        friend_id: i32,
    ) -> Result<(), DbErr> {
        // Create a transaction to ensure both friendships are created or none
        let txn = self.db.begin().await?;

        // Create friendship from user to friend
        let friendship1 = FriendshipActiveModel {
            user_id: sea_orm::Set(user_id),
            friend_id: sea_orm::Set(friend_id),
            ..Default::default()
        };

        // Create friendship from friend to user
        let friendship2 = FriendshipActiveModel {
            user_id: sea_orm::Set(friend_id),
            friend_id: sea_orm::Set(user_id),
            ..Default::default()
        };

        friendship1.insert(&txn).await?;
        friendship2.insert(&txn).await?;

        txn.commit().await?;
        Ok(())
    }
}
