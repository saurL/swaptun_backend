pub use sea_orm_migration::prelude::*;

mod deezer_token_29_04_2025;
mod m20250319_093000_create_tbl_users;
mod music29_04_2025;
mod music_playlist;
mod playlist29_04_2025;
mod playlist_origin_29_04_2025;
mod spotify_code_29_04_2025;
mod spotify_token_29_04_2025;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250319_093000_create_tbl_users::Migration),
            Box::new(deezer_token_29_04_2025::Migration),
            Box::new(spotify_code_29_04_2025::Migration),
            Box::new(playlist29_04_2025::Migration),
            Box::new(music29_04_2025::Migration),
            Box::new(music_playlist::Migration),
            Box::new(spotify_token_29_04_2025::Migration),
            Box::new(playlist_origin_29_04_2025::Migration),
        ]
    }
}
