pub use sea_orm_migration::prelude::*;

mod m2025_03_19_create_tbl_users;
mod m2025_04_29_add_playlist_origin_id;
mod m2025_04_29_create_deezer_token_table;
mod m2025_04_29_create_music_playlist_table;
mod m2025_04_29_create_music_table;
mod m2025_04_29_create_playlist_origin_enum;
mod m2025_04_29_create_playlist_table;
mod m2025_04_29_create_spotify_code_table;
mod m2025_04_29_create_spotify_token_table;
mod m2025_06_18_create_user_info_table;
mod m2025_06_28_create_youtube_token_table;
mod m2025_08_03_create_fcm_token_table;
mod m2025_08_14_enable_fuzzy_search;
mod m2025_08_16_create_friendships_table;
mod m2025_08_16_create_shared_playlist_table;
mod m2025_09_10_create_apple_token_table;
mod m2025_10_05_add_shared_by_to_shared_playlist;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2025_03_19_create_tbl_users::Migration),
            Box::new(m2025_04_29_create_playlist_table::Migration),
            Box::new(m2025_04_29_create_deezer_token_table::Migration),
            Box::new(m2025_04_29_create_music_table::Migration),
            Box::new(m2025_04_29_create_music_playlist_table::Migration),
            Box::new(m2025_04_29_create_spotify_token_table::Migration),
            Box::new(m2025_04_29_create_playlist_origin_enum::Migration),
            Box::new(m2025_06_18_create_user_info_table::Migration),
            Box::new(m2025_06_28_create_youtube_token_table::Migration),
            Box::new(m2025_08_03_create_fcm_token_table::Migration),
            Box::new(m2025_08_14_enable_fuzzy_search::Migration),
            Box::new(m2025_08_16_create_friendships_table::Migration),
            Box::new(m2025_08_16_create_shared_playlist_table::Migration),
            Box::new(m2025_09_10_create_apple_token_table::Migration),
            Box::new(m2025_04_29_add_playlist_origin_id::Migration),
            Box::new(m2025_04_29_create_spotify_code_table::Migration),
            Box::new(m2025_10_05_add_shared_by_to_shared_playlist::Migration),
        ]
    }
}
