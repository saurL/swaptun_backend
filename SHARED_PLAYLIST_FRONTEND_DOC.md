# Documentation Frontend - Playlists Partagées et Notifications

## Vue d'ensemble

Le système de partage de playlists a été amélioré pour inclure des informations sur qui a partagé la playlist et quand. Les notifications de partage incluent maintenant également ces informations.

## Changements API

### 1. Endpoint GET /api/playlists/shared - Récupérer les playlists partagées

**Headers requis:**
- `Authorization: Bearer <jwt_token>`

**Réponse (200 OK):**
```json
[
  {
    "id": 123,
    "playlist": {
      "id": 456,
      "user_id": 789,
      "name": "Ma Super Playlist",
      "description": "Description de la playlist",
      "origin": "Spotify",
      "created_on": "2025-10-05T10:30:00Z",
      "updated_on": "2025-10-05T10:30:00Z",
      "origin_id": "spotify_playlist_id"
    },
    "shared_by": {
      "id": 789,
      "username": "john_doe",
      "first_name": "John",
      "last_name": "Doe"
    },
    "shared_at": "2025-10-05T10:30:00Z"
  }
]
```

**Champs de la réponse:**

| Champ | Type | Description |
|-------|------|-------------|
| `id` | integer | ID unique du partage |
| `playlist` | object | Objet playlist complet |
| `shared_by` | object | Informations sur l'utilisateur qui a partagé |
| `shared_by.id` | integer | ID de l'utilisateur qui a partagé |
| `shared_by.username` | string | Nom d'utilisateur |
| `shared_by.first_name` | string | Prénom |
| `shared_by.last_name` | string | Nom de famille |
| `shared_at` | datetime | Date et heure du partage (ISO 8601 avec timezone UTC) |

### 2. Endpoint POST /api/playlists/{id}/share - Partager une playlist

**Headers requis:**
- `Authorization: Bearer <jwt_token>`

**Paramètres:**
- `{id}`: ID de la playlist à partager (integer)

**Body de la requête:**
```json
{
  "user_id": 123
}
```

**Réponse (204 No Content):**
Aucun contenu retourné en cas de succès.

**Comportement:**
1. L'utilisateur authentifié (celui qui fait la requête) est automatiquement enregistré comme "partagé par"
2. Une notification est envoyée à l'utilisateur avec qui la playlist est partagée
3. La notification contient les informations du partageur

## Notifications de Partage

### Structure de la notification

Lorsqu'une playlist est partagée, une notification Firebase Cloud Messaging est envoyée avec les données suivantes :

**Notification:**
```json
{
  "title": "New Shared Playlist",
  "body": "John Doe shared the playlist 'Ma Super Playlist' with you"
}
```

**Data payload:**
```json
{
  "type": "playlist_shared",
  "playlist_id": "456",
  "playlist_name": "Ma Super Playlist",
  "shared_by_id": "789",
  "shared_by_username": "john_doe",
  "shared_by_name": "John Doe",
  "route": "/home/shared"
}
```

**Champs du data payload:**

| Champ | Type | Description |
|-------|------|-------------|
| `type` | string | Toujours "playlist_shared" pour ce type de notification |
| `playlist_id` | string | ID de la playlist partagée |
| `playlist_name` | string | Nom de la playlist |
| `shared_by_id` | string | ID de l'utilisateur qui a partagé |
| `shared_by_username` | string | Nom d'utilisateur du partageur |
| `shared_by_name` | string | Nom complet du partageur (prénom + nom) |
| `route` | string | Route vers laquelle naviguer (`/home/shared`) |

## Exemples d'implémentation Frontend

### Afficher les playlists partagées

```typescript
interface SharedBy {
  id: number;
  username: string;
  first_name: string;
  last_name: string;
}

interface SharedPlaylist {
  id: number;
  playlist: Playlist;
  shared_by: SharedBy;
  shared_at: string; // ISO 8601 datetime
}

// Récupérer les playlists partagées
const getSharedPlaylists = async (): Promise<SharedPlaylist[]> => {
  const response = await fetch('/api/playlists/shared', {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    throw new Error('Failed to fetch shared playlists');
  }

  return await response.json();
};

// Composant React exemple
const SharedPlaylistsList: React.FC = () => {
  const [sharedPlaylists, setSharedPlaylists] = useState<SharedPlaylist[]>([]);

  useEffect(() => {
    getSharedPlaylists().then(setSharedPlaylists);
  }, []);

  return (
    <div>
      <h2>Playlists partagées avec moi</h2>
      {sharedPlaylists.map((shared) => (
        <div key={shared.id} className="shared-playlist-card">
          <h3>{shared.playlist.name}</h3>
          <p>{shared.playlist.description}</p>
          <div className="shared-info">
            <span>Partagée par: {shared.shared_by.first_name} {shared.shared_by.last_name}</span>
            <span>Le: {new Date(shared.shared_at).toLocaleDateString()}</span>
          </div>
        </div>
      ))}
    </div>
  );
};
```

### Partager une playlist

```typescript
const sharePlaylist = async (playlistId: number, userId: number) => {
  const response = await fetch(`/api/playlists/${playlistId}/share`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      user_id: userId,
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to share playlist');
  }
};

// Composant React exemple
const SharePlaylistButton: React.FC<{ playlistId: number }> = ({ playlistId }) => {
  const [selectedUserId, setSelectedUserId] = useState<number | null>(null);

  const handleShare = async () => {
    if (!selectedUserId) return;

    try {
      await sharePlaylist(playlistId, selectedUserId);
      alert('Playlist partagée avec succès!');
    } catch (error) {
      console.error('Error sharing playlist:', error);
      alert('Erreur lors du partage');
    }
  };

  return (
    <div>
      <select onChange={(e) => setSelectedUserId(Number(e.target.value))}>
        <option value="">Sélectionner un ami</option>
        {/* Liste des amis */}
      </select>
      <button onClick={handleShare} disabled={!selectedUserId}>
        Partager
      </button>
    </div>
  );
};
```

### Gérer les notifications FCM

```typescript
// Configuration Firebase Cloud Messaging
import { getMessaging, onMessage } from 'firebase/messaging';

const messaging = getMessaging();

// Écouter les notifications en arrière-plan
onMessage(messaging, (payload) => {
  console.log('Message received:', payload);

  if (payload.data?.type === 'playlist_shared') {
    const {
      playlist_id,
      playlist_name,
      shared_by_id,
      shared_by_username,
      shared_by_name,
      route,
    } = payload.data;

    // Afficher une notification dans l'app
    showInAppNotification({
      title: payload.notification?.title || 'Nouvelle playlist partagée',
      message: payload.notification?.body || `${shared_by_name} a partagé "${playlist_name}" avec vous`,
      onClick: () => {
        // Naviguer vers la page des playlists partagées
        router.push(route);
      },
      data: {
        playlistId: Number(playlist_id),
        sharedById: Number(shared_by_id),
        sharedByUsername,
        sharedByName,
      },
    });
  }
});
```

## Migration Base de Données

Une nouvelle migration a été ajoutée pour supporter ces fonctionnalités :

**Migration:** `2025_10_05_add_shared_by_to_shared_playlist.rs`

**Changements:**
- Ajout de la colonne `shared_by_user_id` à la table `shared_playlist`
- Foreign key vers la table `tbl_users`

**Note:** Cette migration sera appliquée automatiquement au démarrage du serveur.

## Modèle de Données

### Table `shared_playlist`

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | integer | Clé primaire |
| `user_id` | integer | Utilisateur qui reçoit le partage |
| `playlist_id` | integer | Playlist partagée |
| `shared_by_user_id` | integer | Utilisateur qui a partagé (nouveau) |
| `created_on` | timestamp | Date de création du partage |

## Notes Importantes

1. **Dates avec timezone**: Toutes les dates retournées par l'API sont au format ISO 8601 avec timezone UTC (ex: `2025-10-05T10:30:00Z`)

2. **Authentification requise**: Tous les endpoints nécessitent un token JWT valide

3. **Notification asynchrone**: La notification est envoyée de manière asynchrone. L'endpoint retourne immédiatement après avoir créé le partage, même si la notification échoue

4. **Gestion d'erreurs**:
   - `401 Unauthorized`: Token invalide ou manquant
   - `404 Not Found`: Playlist ou utilisateur non trouvé
   - `500 Internal Server Error`: Erreur serveur

5. **Éviter les doublons**: Si une playlist est déjà partagée avec un utilisateur, le backend retourne succès sans créer de doublon
