use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "friendships")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(comment = "User who initiated the friendship")]
    pub user_id: i32,
    #[sea_orm(comment = "User who is the friend")]
    pub friend_id: i32,
    pub created_on: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::FriendId",
        to = "super::user::Column::Id"
    )]
    Friend,
}

impl ActiveModelBehavior for ActiveModel {}
