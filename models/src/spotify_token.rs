use serde::{Deserialize, Serialize};

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tbl_spotify_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub created_on: DateTime,
    pub updated_on: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::user::Entity",
        from = "Column::UserId",
        to = "crate::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
// `Related` trait has to be implemented by hand

impl Related<crate::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
