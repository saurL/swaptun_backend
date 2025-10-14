use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{prelude::*, DeleteResult, Order, QueryOrder};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QuerySelect, Set,
};
use std::sync::Arc;
use swaptun_models::{
    FriendshipColumn, FriendshipEntity, UserActiveModel, UserColumn, UserEntity, UserModel,
};

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

    pub async fn soft_delete(
        &self,
        id: i32,
        now: DateTimeWithTimeZone,
    ) -> Result<Option<UserModel>, DbErr> {
        let user = self.find_by_id(id).await?;
        let now = chrono::Utc::now().fixed_offset();
        if let Some(user) = user {
            let mut active_model: UserActiveModel = user.into();
            active_model.deleted_on = Set(Some(now));
            active_model.updated_on = Set(now);

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn restore(
        &self,
        id: i32,
        now: DateTimeWithTimeZone,
    ) -> Result<Option<UserModel>, DbErr> {
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

    pub async fn search_in_friends(
        &self,
        user_id: i32,
        search_term: Option<String>,
        search_fields: Option<UserColumn>,
        include_deleted: bool,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<UserModel>, DbErr> {
        let friend_ids = self.get_friend_ids(user_id).await?;

        let query = self
            .build_search_user_query(
                user_id,
                UserEntity::find().filter(UserColumn::Id.is_in(friend_ids)),
                &search_term,
                &search_fields,
                &include_deleted,
                &false,
                &false, // exclude_self is always false for friends search (user can't be their own friend)
            )
            .await?
            .limit(limit.unwrap_or(u64::MAX))
            .offset(offset.unwrap_or(0));

        query.all(self.db.as_ref()).await
    }

    pub async fn search_users(
        &self,
        user_id: i32,
        search_term: Option<String>,
        search_fields: Option<UserColumn>,
        include_deleted: bool,
        limit: Option<u64>,
        offset: Option<u64>,
        friend_priority: bool,
        exclude_friends: bool,
        exclude_self: bool,
    ) -> Result<Vec<UserModel>, DbErr> {
        let query = self
            .build_search_user_query(
                user_id,
                UserEntity::find(),
                &search_term,
                &search_fields,
                &include_deleted,
                &exclude_friends,
                &exclude_self,
            )
            .await?
            .limit(limit.unwrap_or(u64::MAX))
            .offset(offset.unwrap_or(0));

        let mut users = query.all(self.db.as_ref()).await?;
        if friend_priority {
            // Prioritize friends in the results
            let mut friends = self
                .search_in_friends(
                    user_id,
                    search_term,
                    search_fields,
                    include_deleted,
                    limit,
                    offset,
                )
                .await?;
            friends.extend(users);
            let mut seen_ids = std::collections::HashSet::new();

            friends.retain(|x| seen_ids.insert(x.id));
            users = friends;
        }

        Ok(users)
    }

    fn calculate_threshold(&self, search_term: &str) -> f64 {
        // Implement your threshold calculation logic here
        // For example, you might want to use a fixed threshold or a dynamic one based on the search term
        let len = search_term.len();
        (0.1 + (len - 1) as f64 * 0.035).min(0.55)
    }

    pub async fn find_friends(&self, user_id: i32) -> Result<Vec<UserModel>, DbErr> {
        let friend_ids = self.get_friend_ids(user_id).await?;
        UserEntity::find()
            .filter(UserColumn::Id.is_in(friend_ids))
            .all(&*self.db)
            .await
    }

    async fn get_friend_ids(&self, user_id: i32) -> Result<Vec<i32>, DbErr> {
        // Get all friendships where user_id added someone
        let friendships_initiated = FriendshipEntity::find()
            .filter(FriendshipColumn::UserId.eq(user_id))
            .all(&*self.db)
            .await?;

        let mut mutual_friend_ids = Vec::new();

        // For each friendship, check if the reverse relationship exists (mutual)
        for friendship in friendships_initiated {
            let reverse_exists = FriendshipEntity::find()
                .filter(
                    FriendshipColumn::UserId
                        .eq(friendship.friend_id)
                        .and(FriendshipColumn::FriendId.eq(user_id)),
                )
                .one(&*self.db)
                .await?;

            // Only include if the friendship is mutual (both added each other)
            if reverse_exists.is_some() {
                mutual_friend_ids.push(friendship.friend_id);
            }
        }

        Ok(mutual_friend_ids)
    }

    async fn build_search_user_query(
        &self,
        user_id: i32,
        base_query: Select<UserEntity>,
        search_term: &Option<String>,
        search_field: &Option<UserColumn>,
        include_deleted: &bool,
        exclude_friends: &bool,
        exclude_self: &bool,
    ) -> Result<Select<UserEntity>, DbErr> {
        let mut query = base_query;

        if !include_deleted {
            query = query.filter(UserColumn::DeletedOn.is_null());
        }

        if let (Some(field), Some(search_term)) = (search_field, search_term) {
            let field_str = match field {
                UserColumn::Username => "username",
                UserColumn::FirstName => "first_name",
                UserColumn::LastName => "last_name",
                UserColumn::Email => "email",
                _ => return Ok(query), // ignore invalid field
            };
            let threshold = self.calculate_threshold(&search_term);
            let condition = Expr::cust(format!(
                "similarity({}, '{}') > {}",
                field_str, search_term, threshold
            ));
            let order_by = Expr::cust(format!("similarity({}, '{}')", field_str, search_term));
            query = query.filter(condition).order_by(order_by, Order::Desc);
        }
        if *exclude_friends {
            let ids = self.get_friend_ids(user_id.clone()).await?;
            query = query.filter(UserColumn::Id.is_not_in(ids));
        }
        if *exclude_self {
            query = query.filter(UserColumn::Id.ne(user_id));
        }

        Ok(query)
    }
}
