# Documentation des changements API - Envoi de playlist vers les plateformes

## Résumé des changements

L'endpoint d'envoi de playlist vers les plateformes musicales a été standardisé pour retourner un objet DTO unifié contenant l'ID de la playlist créée sur la plateforme cible, quelle que soit la plateforme (Spotify, Apple Music, YouTube Music).

## Endpoint modifié

### POST `/api/playlists/{id}/send`

**Description**: Envoie une playlist locale vers une plateforme musicale externe (Spotify, Apple Music, ou YouTube Music).

**Authentification**: Requise (JWT Token)

**Paramètres d'URL**:
- `{id}` (integer): L'ID de la playlist locale à envoyer

**Corps de la requête** (JSON):
```json
{
  "destination": "Spotify" | "AppleMusic" | "YoutubeMusic"
}
```

**Réponse en cas de succès** (Status: 200 OK):
```json
{
  "platform": "Spotify" | "AppleMusic" | "YoutubeMusic",
  "playlist_id": "string"
}
```

**Champs de la réponse**:
- `platform` (PlaylistOrigin enum): La plateforme vers laquelle la playlist a été envoyée
- `playlist_id` (string): L'identifiant unique de la playlist créée sur la plateforme cible

## Exemples d'utilisation

### Exemple 1: Envoyer vers Spotify

**Requête**:
```http
POST /api/playlists/42/send
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "destination": "Spotify"
}
```

**Réponse**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "platform": "Spotify",
  "playlist_id": "37i9dQZF1DXcBWIGoYBM5M"
}
```

### Exemple 2: Envoyer vers Apple Music

**Requête**:
```http
POST /api/playlists/42/send
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "destination": "AppleMusic"
}
```

**Réponse**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "platform": "AppleMusic",
  "playlist_id": "pl.u-8aAVZKoCWjeLjy"
}
```

### Exemple 3: Envoyer vers YouTube Music

**Requête**:
```http
POST /api/playlists/42/send
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "destination": "YoutubeMusic"
}
```

**Réponse**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "platform": "YoutubeMusic",
  "playlist_id": "VLPLK1Hw8AmjVtU0Z3a"
}
```

## Cas d'usage Frontend

### 1. Afficher un lien vers la playlist créée

Après avoir envoyé une playlist, vous pouvez utiliser l'ID retourné pour créer un lien direct vers la playlist sur la plateforme:

```typescript
// Fonction utilitaire pour générer les URLs de playlist
function getPlaylistUrl(platform: string, playlistId: string): string {
  switch (platform) {
    case 'Spotify':
      return `https://open.spotify.com/playlist/${playlistId}`;
    case 'AppleMusic':
      return `https://music.apple.com/library/playlist/${playlistId}`;
    case 'YoutubeMusic':
      return `https://music.youtube.com/playlist?list=${playlistId}`;
    default:
      return '';
  }
}

// Exemple d'utilisation après l'envoi
async function sendPlaylist(playlistId: number, destination: string) {
  try {
    const response = await fetch(`/api/playlists/${playlistId}/send`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ destination }),
    });

    if (response.ok) {
      const data = await response.json();
      const playlistUrl = getPlaylistUrl(data.platform, data.playlist_id);

      // Afficher un message de succès avec le lien
      showSuccessMessage(
        `Playlist envoyée avec succès vers ${data.platform}!`,
        playlistUrl
      );
    }
  } catch (error) {
    console.error('Erreur lors de l\'envoi de la playlist:', error);
  }
}
```

### 2. Sauvegarder l'ID de playlist pour référence future

Vous pouvez stocker l'ID de playlist retourné pour permettre à l'utilisateur de retrouver facilement ses playlists exportées:

```typescript
interface ExportedPlaylist {
  localPlaylistId: number;
  platform: string;
  platformPlaylistId: string;
  exportedAt: Date;
}

async function sendAndSavePlaylist(playlistId: number, destination: string) {
  const response = await fetch(`/api/playlists/${playlistId}/send`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ destination }),
  });

  if (response.ok) {
    const data = await response.json();

    // Sauvegarder dans le localStorage ou votre state management
    const exportedPlaylist: ExportedPlaylist = {
      localPlaylistId: playlistId,
      platform: data.platform,
      platformPlaylistId: data.playlist_id,
      exportedAt: new Date(),
    };

    saveExportedPlaylist(exportedPlaylist);
    return exportedPlaylist;
  }
}
```

### 3. Afficher un indicateur de progression

```typescript
async function sendPlaylistWithProgress(
  playlistId: number,
  destination: string,
  onProgress: (message: string) => void
) {
  onProgress(`Envoi de la playlist vers ${destination}...`);

  try {
    const response = await fetch(`/api/playlists/${playlistId}/send`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ destination }),
    });

    if (response.ok) {
      const data = await response.json();
      onProgress(`✓ Playlist créée sur ${data.platform} avec succès!`);
      return data;
    } else {
      onProgress(`✗ Échec de l'envoi vers ${destination}`);
      throw new Error('Failed to send playlist');
    }
  } catch (error) {
    onProgress(`✗ Erreur: ${error.message}`);
    throw error;
  }
}
```

## Gestion des erreurs

Les erreurs possibles incluent:

**401 Unauthorized**: Token JWT manquant ou invalide
```json
{
  "error": "Unauthorized",
  "message": "No authentication token found"
}
```

**404 Not Found**: Playlist introuvable
```json
{
  "error": "NotFound",
  "message": "Playlist not found"
}
```

**500 Internal Server Error**:
- L'utilisateur n'est pas connecté à la plateforme cible
- Erreur lors de la création de la playlist sur la plateforme
- Deezer n'est pas encore supporté

Exemple de gestion des erreurs:
```typescript
async function sendPlaylistSafely(playlistId: number, destination: string) {
  try {
    const response = await fetch(`/api/playlists/${playlistId}/send`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ destination }),
    });

    if (!response.ok) {
      const errorData = await response.json();

      switch (response.status) {
        case 401:
          throw new Error('Vous devez être connecté pour effectuer cette action');
        case 404:
          throw new Error('Playlist introuvable');
        case 500:
          if (destination === 'Deezer') {
            throw new Error('Deezer n\'est pas encore supporté');
          }
          throw new Error(`Assurez-vous d'être connecté à ${destination}`);
        default:
          throw new Error(errorData.message || 'Une erreur est survenue');
      }
    }

    return await response.json();
  } catch (error) {
    console.error('Erreur lors de l\'envoi:', error);
    throw error;
  }
}
```

## Types TypeScript recommandés

```typescript
// Types pour l'API
type PlaylistOrigin = 'Spotify' | 'AppleMusic' | 'YoutubeMusic' | 'Deezer' | 'Local';

interface SendPlaylistRequest {
  destination: PlaylistOrigin;
}

interface SendPlaylistResponse {
  platform: PlaylistOrigin;
  playlist_id: string;
}

// Service API
class PlaylistApiService {
  async sendToplatform(
    playlistId: number,
    destination: PlaylistOrigin
  ): Promise<SendPlaylistResponse> {
    const response = await fetch(`/api/playlists/${playlistId}/send`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.getToken()}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ destination } as SendPlaylistRequest),
    });

    if (!response.ok) {
      throw new Error(`Failed to send playlist: ${response.statusText}`);
    }

    return response.json();
  }

  private getToken(): string {
    // Récupérer le token JWT depuis votre state management
    return localStorage.getItem('jwt_token') || '';
  }
}
```

## Notes importantes

1. **Authentification obligatoire**: L'utilisateur doit être authentifié et connecté à la plateforme cible avant d'envoyer une playlist
2. **Support des plateformes**:
   - ✅ Spotify: Complètement supporté
   - ✅ Apple Music: Complètement supporté
   - ✅ YouTube Music: Complètement supporté
   - ❌ Deezer: Non supporté actuellement (retourne une erreur 500)
3. **Format des IDs**: Les IDs de playlist varient selon la plateforme mais sont toujours retournés sous forme de chaîne de caractères
4. **Correspondance des titres**: Le système fait une correspondance fuzzy des titres/artistes, donc certaines musiques peuvent ne pas être trouvées sur la plateforme cible

## Migration depuis l'ancienne API

**Avant** (retournait une chaîne de caractères):
```typescript
const result: string = await sendPlaylist(42, 'Spotify');
// result: "Playlist sent to Spotify successfully with ID: 37i9dQZF1DXcBWIGoYBM5M"
```

**Maintenant** (retourne un objet structuré):
```typescript
const result: SendPlaylistResponse = await sendPlaylist(42, 'Spotify');
// result: { platform: "Spotify", playlist_id: "37i9dQZF1DXcBWIGoYBM5M" }
```

Pour une migration en douceur, vous pouvez adapter votre code existant:
```typescript
// Ancienne logique qui parsait la chaîne de caractères
const oldResult = "Playlist sent to Spotify successfully with ID: 37i9dQZF1DXcBWIGoYBM5M";
const playlistId = oldResult.split("ID: ")[1]; // Fragile!

// Nouvelle logique avec l'objet structuré
const newResult = { platform: "Spotify", playlist_id: "37i9dQZF1DXcBWIGoYBM5M" };
const playlistId = newResult.playlist_id; // Robuste et typé!
```
