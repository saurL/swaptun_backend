# Migration des Timestamps vers TIMESTAMPTZ

## Vue d'ensemble

Tous les modèles de données ont été mis à jour pour utiliser `DateTimeWithTimeZone` au lieu de `DateTime` ou `NaiveDateTime`. Cela permet de stocker les dates avec leur timezone en base de données PostgreSQL (type `TIMESTAMPTZ`).

## Changements Effectués

### 1. Migration Base de Données

**Migration:** [2025_10_05_convert_timestamps_to_timestamptz.rs](migrations/src/m2025_10_05_convert_timestamps_to_timestamptz.rs)

Cette migration convertit toutes les colonnes `TIMESTAMP` en `TIMESTAMPTZ` pour les tables suivantes :

- `tbl_users` : `created_on`, `updated_on`, `deleted_on`
- `playlist` : `created_on`, `updated_on`
- `spotify_token` : `expires_at`, `created_on`, `updated_on`
- `spotify_code` : `created_on`, `updated_on`
- `deezer_token` : `created_on`, `updated_on`
- `apple_token` : `created_on`, `updated_on`
- `youtube_token` : `created_on`, `updated_on`
- `fcm_token` : `created_on`, `updated_on`
- `friendship` : `created_on`
- `shared_playlist` : `created_on`

**Rollback:** La migration `down()` restaure les colonnes en `TIMESTAMP` si nécessaire.

### 2. Modèles Mis à Jour

Tous les modèles SeaORM ont été modifiés :

**Avant:**

```rust
use sea_orm::{entity::prelude::*, sqlx::types::chrono::NaiveDateTime};

pub struct Model {
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
}
```

**Après:**

```rust
use sea_orm::entity::prelude::*;

pub struct Model {
    pub created_on: DateTimeWithTimeZone,
    pub updated_on: DateTimeWithTimeZone,
}
```

**Modèles affectés:**

- [user.rs](models/src/user.rs) - `created_on`, `updated_on`, `deleted_on`
- [playlist.rs](models/src/playlist.rs) - `created_on`, `updated_on`
- [spotify_token.rs](models/src/spotify_token.rs) - `expires_at`, `created_on`, `updated_on`
- [spotify_code.rs](models/src/spotify_code.rs) - `created_on`, `updated_on`
- [deezer_token.rs](models/src/deezer_token.rs) - `created_on`, `updated_on`
- [apple_token.rs](models/src/apple_token.rs) - `created_on`, `updated_on`
- [youtube_token.rs](models/src/youtube_token.rs) - `created_on`, `updated_on`
- [fcm_token.rs](models/src/fcm_token.rs) - `created_on`, `updated_on`
- [friendship.rs](models/src/friendship.rs) - `created_on`
- [shared_playlist.rs](models/src/shared_playlist.rs) - `created_on`

### 3. Type Mapping

**SeaORM Type Mapping:**

| Rust Type                | PostgreSQL Type | Description                   |
| ------------------------ | --------------- | ----------------------------- |
| `DateTimeWithTimeZone`   | `TIMESTAMPTZ`   | Timestamp avec timezone (UTC) |
| `DateTime` (ancien)      | `TIMESTAMP`     | Timestamp sans timezone       |
| `NaiveDateTime` (ancien) | `TIMESTAMP`     | Timestamp sans timezone       |

**Sérialisation JSON:**

Les `DateTimeWithTimeZone` sont automatiquement sérialisés en ISO 8601 avec timezone UTC :

```json
{
  "created_on": "2025-10-05T14:30:00Z",
  "updated_on": "2025-10-05T14:30:00Z"
}
```

### 4. Compatibilité avec les DTOs

Les DTOs utilisent `DateTime<Utc>` de chrono, qui est compatible avec `DateTimeWithTimeZone` :

```rust
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct SharedPlaylistResponse {
    pub shared_at: DateTime<Utc>, // Compatible avec DateTimeWithTimeZone
}
```

La conversion est automatique lors de la sérialisation/désérialisation.

## Impact Frontend

### Format des Dates

Toutes les dates retournées par l'API sont maintenant au format ISO 8601 avec timezone UTC explicite :

**Avant:**

```json
{
  "created_on": "2025-10-05T14:30:00" // Ambiguë
}
```

**Après:**

```json
{
  "created_on": "2025-10-05T14:30:00Z" // UTC explicite
}
```

### Parsing Frontend

**JavaScript/TypeScript:**

```typescript
// Les dates sont automatiquement parsées avec timezone
const createdOn = new Date(response.created_on); // 2025-10-05T14:30:00Z
console.log(createdOn.toLocaleString()); // Converti dans la timezone locale
```

**React Example:**

```tsx
const formatDate = (dateString: string) => {
  const date = new Date(dateString);
  return date.toLocaleString("fr-FR", {
    dateStyle: "medium",
    timeStyle: "short",
    timeZone: "Europe/Paris", // Optionnel: forcer une timezone
  });
};

// Utilisation
<span>{formatDate(playlist.created_on)}</span>;
```

## Migrations Futures

Pour les nouvelles migrations, utiliser `timestamp_with_time_zone()` :

```rust
use sea_orm_migration::prelude::*;

manager
    .create_table(
        Table::create()
            .table(MyTable::Table)
            .col(
                ColumnDef::new(MyTable::CreatedOn)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp_with_time_zone())
            )
            .to_owned()
    )
    .await
```

## Ordre d'Exécution des Migrations

Les migrations s'exécutent dans cet ordre :

1. Toutes les migrations de création de tables (avec `TIMESTAMP`)
2. `2025_10_05_add_apple_music_to_playlist_origin` - Ajout enum AppleMusic
3. `2025_10_05_add_shared_by_to_shared_playlist` - Ajout shared_by_user_id
4. **`2025_10_05_convert_timestamps_to_timestamptz`** - Conversion TIMESTAMPTZ

## Tests et Validation

### Vérifier la Migration

```sql
-- Vérifier les types de colonnes
SELECT
    table_name,
    column_name,
    data_type
FROM information_schema.columns
WHERE table_schema = 'public'
AND data_type IN ('timestamp without time zone', 'timestamp with time zone')
ORDER BY table_name, column_name;
```

**Résultat attendu:** Toutes les colonnes de date doivent être `timestamp with time zone`.

### Test API

```bash
# Récupérer une playlist partagée
curl -H "Authorization: Bearer <token>" \
  http://localhost:8000/api/playlists/shared

# Réponse attendue
{
  "id": 1,
  "playlist": {...},
  "shared_by": {...},
  "shared_at": "2025-10-05T14:30:00Z"  # Avec 'Z' pour UTC
}
```

## Rollback

En cas de problème, la migration peut être annulée :

```bash
# Rollback de la dernière migration
sea-orm-cli migrate down

# Ou manuellement en SQL
ALTER TABLE tbl_users
  ALTER COLUMN created_on TYPE timestamp,
  ALTER COLUMN updated_on TYPE timestamp;

-- Répéter pour toutes les tables
```

## Notes Importantes

1. **PostgreSQL stocke toujours en UTC:** Même avec `TIMESTAMPTZ`, PostgreSQL stocke en UTC et convertit automatiquement

2. **Pas de perte de données:** La conversion `TIMESTAMP → TIMESTAMPTZ` est safe car PostgreSQL assume UTC pour les timestamps sans timezone

3. **Performances:** Aucun impact sur les performances, `TIMESTAMPTZ` et `TIMESTAMP` ont la même taille en mémoire

4. **Compatibilité:** Les anciennes données sont préservées et automatiquement converties en UTC

5. **SeaORM:** `DateTimeWithTimeZone` utilise `chrono::DateTime<FixedOffset>` en interne, compatible avec JSON et serde
