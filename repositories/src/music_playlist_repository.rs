use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    ModelTrait, QueryFilter,
};
use std::sync::Arc;
use swaptun_models::{
    MusicEntity, MusicModel, MusicPlaylistActiveModel, MusicPlaylistColumn, MusicPlaylistEntity,
    MusicPlaylistModel, PlaylistModel,
};

#[derive(Clone)]
pub struct MusicPlaylistRepository {
    db: Arc<DatabaseConnection>,
}

impl MusicPlaylistRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        music_playlist: MusicPlaylistActiveModel,
    ) -> Result<MusicPlaylistModel, DbErr> {
        music_playlist.insert(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        MusicPlaylistEntity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
    }

    pub async fn delete_relation(
        &self,
        playlist: &PlaylistModel,
        music: &MusicModel,
    ) -> Result<DeleteResult, DbErr> {
        MusicPlaylistEntity::delete_many()
            .filter(
                MusicPlaylistColumn::PlaylistId.eq(playlist.id).and(
                    MusicPlaylistColumn::MusicAlbum.eq(music.album.clone()).and(
                        MusicPlaylistColumn::MusicArtist
                            .eq(music.artist.clone())
                            .and(MusicPlaylistColumn::MusicTitle.eq(music.title.clone())),
                    ),
                ),
            )
            .exec(self.db.as_ref())
            .await
    }

    pub async fn find_musics_by_playlist(
        &self,
        playlist: PlaylistModel,
    ) -> Result<Vec<MusicModel>, DbErr> {
        playlist
            .find_related(MusicEntity)
            .all(self.db.as_ref())
            .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<MusicPlaylistModel>, DbErr> {
        MusicPlaylistEntity::find_by_id(id)
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_relation(
        &self,
        playlist_id: PlaylistModel,
        music_id: MusicModel,
    ) -> Result<Option<MusicPlaylistModel>, DbErr> {
        MusicPlaylistEntity::find()
            .filter(
                MusicPlaylistColumn::PlaylistId.eq(playlist_id.id).and(
                    MusicPlaylistColumn::MusicAlbum.eq(music_id.album).and(
                        MusicPlaylistColumn::MusicArtist
                            .eq(music_id.artist)
                            .and(MusicPlaylistColumn::MusicTitle.eq(music_id.title)),
                    ),
                ),
            )
            .one(self.db.as_ref())
            .await
    }

    pub async fn delete_by_playlist_id(&self, playlist_id: i32) -> Result<DeleteResult, DbErr> {
        MusicPlaylistEntity::delete_many()
            .filter(MusicPlaylistColumn::PlaylistId.eq(playlist_id))
            .exec(self.db.as_ref())
            .await
    }
}
