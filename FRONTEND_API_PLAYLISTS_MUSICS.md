# Documentation API - Récupération des playlists avec musiques

## Résumé des changements

L'API de récupération des playlists a été améliorée pour permettre d'inclure optionnellement les musiques dans la réponse. Un nouvel endpoint dédié permet également de récupérer uniquement les musiques d'une playlist spécifique.

## Endpoints modifiés et nouveaux

### 1. GET `/api/playlists` - Récupérer les playlists de l'utilisateur

**Description**: Récupère la liste des playlists de l'utilisateur authentifié, avec option d'inclure les musiques.

**Authentification**: Requise (JWT Token)

**Méthode**: GET

**Corps de la requête** (JSON):

```json
{
  "origin": "Spotify" | "AppleMusic" | "YoutubeMusic" | "Deezer" | "Local" | null,
  "include_musics": false | true
}
```

**Paramètres**:

- `origin` (PlaylistOrigin, optionnel): Filtre les playlists par origine/plateforme. Si null, retourne toutes les playlists.
- `include_musics` (boolean, optionnel, défaut: false): Si true, inclut la liste des musiques pour chaque playlist.

#### Réponse sans musiques (include_musics: false ou omis)

**Status**: 200 OK

```json
{
  "vec": [
    {
      "id": 1,
      "name": "Ma playlist cool",
      "description": "Une description",
      "user_id": 42,
      "origin": "Spotify",
      "origin_id": "spotify_playlist_id_123",
      "created_on": "2024-01-15T10:30:00Z",
      "updated_on": "2024-01-15T10:30:00Z"
    },
    {
      "id": 2,
      "name": "Workout Mix",
      "description": null,
      "user_id": 42,
      "origin": "Local",
      "origin_id": "local_2",
      "created_on": "2024-01-16T14:20:00Z",
      "updated_on": "2024-01-16T14:20:00Z"
    }
  ]
}
```

#### Réponse avec musiques (include_musics: true)

**Status**: 200 OK

```json
{
  "playlists": [
    {
      "id": 1,
      "name": "Ma playlist cool",
      "description": "Une description",
      "user_id": 42,
      "origin": "Spotify",
      "origin_id": "spotify_playlist_id_123",
      "created_on": "2024-01-15T10:30:00Z",
      "updated_on": "2024-01-15T10:30:00Z",
      "musics": [
        {
          "title": "Bohemian Rhapsody",
          "artist": "Queen",
          "album": "A Night at the Opera",
          "release_date": "1975-10-31",
          "genre": "Rock"
        },
        {
          "title": "Stairway to Heaven",
          "artist": "Led Zeppelin",
          "album": "Led Zeppelin IV",
          "release_date": "1971-11-08",
          "genre": "Rock"
        }
      ]
    },
    {
      "id": 2,
      "name": "Workout Mix",
      "description": null,
      "user_id": 42,
      "origin": "Local",
      "origin_id": "local_2",
      "created_on": "2024-01-16T14:20:00Z",
      "updated_on": "2024-01-16T14:20:00Z",
      "musics": []
    }
  ]
}
```

### 2. GET `/api/playlists/{id}/musics` - Récupérer les musiques d'une playlist

**Description**: Récupère uniquement la liste des musiques d'une playlist spécifique.

**Authentification**: Requise (JWT Token)

**Méthode**: GET

**Paramètres d'URL**:

- `{id}` (integer): L'ID de la playlist

**Réponse en cas de succès** (Status: 200 OK):

```json
{
  "playlist_id": 1,
  "musics": [
    {
      "title": "Bohemian Rhapsody",
      "artist": "Queen",
      "album": "A Night at the Opera",
      "release_date": "1975-10-31",
      "genre": "Rock"
    },
    {
      "title": "Stairway to Heaven",
      "artist": "Led Zeppelin",
      "album": "Led Zeppelin IV",
      "release_date": "1971-11-08",
      "genre": "Rock"
    }
  ]
}
```

**Champs de la réponse**:

- `playlist_id` (integer): L'ID de la playlist
- `musics` (array): Liste des musiques dans la playlist
  - `title` (string): Titre de la musique
  - `artist` (string): Nom de l'artiste
  - `album` (string): Nom de l'album
  - `release_date` (string): Date de sortie au format ISO 8601 (YYYY-MM-DD)
  - `genre` (string, nullable): Genre musical

## Exemples d'utilisation

### Exemple 1: Récupérer toutes les playlists sans musiques

**Requête**:

```http
GET /api/playlists
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "origin": null,
  "include_musics": false
}
```

**Réponse**:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "vec": [
    {
      "id": 1,
      "name": "Ma playlist cool",
      "description": "Une description",
      "user_id": 42,
      "origin": "Spotify",
      "origin_id": "spotify_playlist_id_123",
      "created_on": "2024-01-15T10:30:00Z",
      "updated_on": "2024-01-15T10:30:00Z"
    }
  ]
}
```

### Exemple 2: Récupérer les playlists Spotify avec leurs musiques

**Requête**:

```http
GET /api/playlists
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "origin": "Spotify",
  "include_musics": true
}
```

**Réponse**:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "playlists": [
    {
      "id": 1,
      "name": "Ma playlist Spotify",
      "description": "Mes meilleures chansons",
      "user_id": 42,
      "origin": "Spotify",
      "origin_id": "spotify_playlist_id_123",
      "created_on": "2024-01-15T10:30:00Z",
      "updated_on": "2024-01-15T10:30:00Z",
      "musics": [
        {
          "title": "Bohemian Rhapsody",
          "artist": "Queen",
          "album": "A Night at the Opera",
          "release_date": "1975-10-31",
          "genre": "Rock"
        }
      ]
    }
  ]
}
```

### Exemple 3: Récupérer les musiques d'une playlist spécifique

**Requête**:

```http
GET /api/playlists/1/musics
Authorization: Bearer <jwt_token>
```

**Réponse**:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "playlist_id": 1,
  "musics": [
    {
      "title": "Bohemian Rhapsody",
      "artist": "Queen",
      "album": "A Night at the Opera",
      "release_date": "1975-10-31",
      "genre": "Rock"
    },
    {
      "title": "Stairway to Heaven",
      "artist": "Led Zeppelin",
      "album": "Led Zeppelin IV",
      "release_date": "1971-11-08",
      "genre": "Rock"
    }
  ]
}
```

## Cas d'usage Frontend

### 1. Afficher la liste des playlists sans charger les musiques (pour une liste rapide)

```typescript
interface Playlist {
  id: number;
  name: string;
  description: string | null;
  user_id: number;
  origin: PlaylistOrigin;
  origin_id: string;
  created_on: string;
  updated_on: string;
}

interface GetPlaylistsResponse {
  vec: Playlist[];
}

async function getPlaylists(origin?: PlaylistOrigin): Promise<Playlist[]> {
  const response = await fetch("/api/playlists", {
    method: "GET",
    headers: {
      Authorization: `Bearer ${getToken()}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      origin: origin || null,
      include_musics: false, // Charge rapidement sans les musiques
    }),
  });

  if (response.ok) {
    const data: GetPlaylistsResponse = await response.json();
    return data.vec;
  }

  throw new Error("Failed to fetch playlists");
}

// Utilisation
const playlists = await getPlaylists(); // Toutes les playlists
const spotifyPlaylists = await getPlaylists("Spotify"); // Seulement Spotify
```

### 2. Charger les playlists avec toutes leurs musiques (pour un affichage détaillé)

```typescript
interface Music {
  title: string;
  artist: string;
  album: string;
  release_date: string;
  genre: string | null;
}

interface Playlistextends Playlist {
  musics: Music[];
}

interface GetPlaylistsWithMusicsResponse {
  playlists: Playlist[];
}

async function getPlaylistsWithMusics(
  origin?: PlaylistOrigin
): Promise<Playlist[]> {
  const response = await fetch('/api/playlists', {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${getToken()}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      origin: origin || null,
      include_musics: true, // Inclut toutes les musiques
    }),
  });

  if (response.ok) {
    const data: GetPlaylistsWithMusicsResponse = await response.json();
    return data.playlists;
  }

  throw new Error('Failed to fetch playlists with musics');
}

// Utilisation dans un composant React
function PlaylistsWithMusicsList() {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);

  useEffect(() => {
    getPlaylistsWithMusics().then(setPlaylists);
  }, []);

  return (
    <div>
      {playlists.map((playlist) => (
        <div key={playlist.id}>
          <h3>{playlist.name}</h3>
          <p>{playlist.description}</p>
          <ul>
            {playlist.musics.map((music, index) => (
              <li key={index}>
                {music.title} - {music.artist} ({music.album})
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}
```

### 3. Charger les musiques à la demande (chargement progressif)

```typescript
interface GetPlaylistMusicsResponse {
  playlist_id: number;
  musics: Music[];
}

async function getPlaylistMusics(playlistId: number): Promise<Music[]> {
  const response = await fetch(`/api/playlists/${playlistId}/musics`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${getToken()}`,
    },
  });

  if (response.ok) {
    const data: GetPlaylistMusicsResponse = await response.json();
    return data.musics;
  }

  throw new Error("Failed to fetch playlist musics");
}

// Utilisation avec chargement à la demande
function PlaylistCard({ playlist }: { playlist: Playlist }) {
  const [musics, setMusics] = useState<Music[] | null>(null);
  const [loading, setLoading] = useState(false);

  const loadMusics = async () => {
    setLoading(true);
    try {
      const data = await getPlaylistMusics(playlist.id);
      setMusics(data);
    } catch (error) {
      console.error("Error loading musics:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="playlist-card">
      <h3>{playlist.name}</h3>
      <button onClick={loadMusics} disabled={loading}>
        {loading ? "Chargement..." : "Voir les musiques"}
      </button>

      {musics && (
        <ul>
          {musics.map((music, index) => (
            <li key={index}>
              {music.title} - {music.artist}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

### 4. Stratégie de chargement optimisée

```typescript
class PlaylistService {
  private token: string;

  constructor(token: string) {
    this.token = token;
  }

  // Méthode 1: Chargement rapide pour l'affichage initial
  async getPlaylistsQuick(origin?: PlaylistOrigin): Promise<Playlist[]> {
    const response = await fetch("/api/playlists", {
      method: "GET",
      headers: {
        Authorization: `Bearer ${this.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        origin: origin || null,
        include_musics: false,
      }),
    });

    const data: GetPlaylistsResponse = await response.json();
    return data.vec;
  }

  // Méthode 2: Chargement complet pour l'export ou l'affichage détaillé
  async getPlaylistsFull(origin?: PlaylistOrigin): Promise<Playlist[]> {
    const response = await fetch("/api/playlists", {
      method: "GET",
      headers: {
        Authorization: `Bearer ${this.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        origin: origin || null,
        include_musics: true,
      }),
    });

    const data: GetPlaylistsWithMusicsResponse = await response.json();
    return data.playlists;
  }

  // Méthode 3: Chargement à la demande d'une seule playlist
  async getPlaylistMusics(playlistId: number): Promise<Music[]> {
    const response = await fetch(`/api/playlists/${playlistId}/musics`, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${this.token}`,
      },
    });

    const data: GetPlaylistMusicsResponse = await response.json();
    return data.musics;
  }
}

// Utilisation avec stratégie de chargement intelligent
function PlaylistsPage() {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [expandedPlaylist, setExpandedPlaylist] = useState<number | null>(null);
  const [musicsCache, setMusicsCache] = useState<Record<number, Music[]>>({});

  const playlistService = new PlaylistService(getToken());

  // Chargement initial rapide
  useEffect(() => {
    playlistService.getPlaylistsQuick().then(setPlaylists);
  }, []);

  // Chargement à la demande quand on expand une playlist
  const handleExpand = async (playlistId: number) => {
    if (expandedPlaylist === playlistId) {
      setExpandedPlaylist(null);
      return;
    }

    setExpandedPlaylist(playlistId);

    // Vérifier le cache
    if (!musicsCache[playlistId]) {
      const musics = await playlistService.getPlaylistMusics(playlistId);
      setMusicsCache((prev) => ({ ...prev, [playlistId]: musics }));
    }
  };

  return (
    <div>
      {playlists.map((playlist) => (
        <div key={playlist.id}>
          <div onClick={() => handleExpand(playlist.id)}>
            <h3>{playlist.name}</h3>
          </div>
          {expandedPlaylist === playlist.id && musicsCache[playlist.id] && (
            <ul>
              {musicsCache[playlist.id].map((music, index) => (
                <li key={index}>
                  {music.title} - {music.artist}
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </div>
  );
}
```

## Gestion des erreurs

**401 Unauthorized**: Token JWT manquant ou invalide

```json
{
  "error": "Unauthorized",
  "message": "No authentication token found"
}
```

**404 Not Found**: Playlist introuvable (pour l'endpoint `/api/playlists/{id}/musics`)

```json
{
  "error": "NotFound",
  "message": "Playlist not found"
}
```

**500 Internal Server Error**: Erreur serveur

```json
{
  "error": "InternalServerError",
  "message": "An error occurred while fetching playlists"
}
```

## Types TypeScript recommandés

```typescript
// Types de base
type PlaylistOrigin = 'Spotify' | 'AppleMusic' | 'YoutubeMusic' | 'Deezer' | 'Local';

interface Music {
  title: string;
  artist: string;
  album: string;
  release_date: string; // Format ISO 8601: YYYY-MM-DD
  genre: string | null;
}

interface Playlist {
  id: number;
  name: string;
  description: string | null;
  user_id: number;
  origin: PlaylistOrigin;
  origin_id: string;
  created_on: string; // Format ISO 8601
  updated_on: string; // Format ISO 8601
}

interface Playlistextends Playlist {
  musics: Music[];
}

// Requêtes
interface GetPlaylistsRequest {
  origin: PlaylistOrigin | null;
  include_musics?: boolean; // Défaut: false
}

// Réponses
interface GetPlaylistsResponse {
  vec: Playlist[];
}

interface GetPlaylistsWithMusicsResponse {
  playlists: Playlist[];
}

interface GetPlaylistMusicsResponse {
  playlist_id: number;
  musics: Music[];
}
```

## Notes importantes

1. **Performance**:

   - Utilisez `include_musics: false` pour un chargement initial rapide
   - Utilisez `include_musics: true` seulement quand vous avez besoin de toutes les musiques de toutes les playlists
   - Utilisez `/api/playlists/{id}/musics` pour charger les musiques à la demande

2. **Compatibilité**:

   - L'ancien comportement est préservé: si `include_musics` est omis ou false, la réponse est identique à avant
   - Les deux types de réponse ont des structures différentes pour éviter toute confusion

3. **Filtrage par origine**:

   - Utilisez `origin: null` pour récupérer toutes les playlists
   - Spécifiez une origine pour filtrer (ex: "Spotify", "AppleMusic", etc.)

4. **Cache côté frontend**:

   - Considérez implémenter un cache pour éviter de recharger les musiques déjà récupérées
   - Invalidez le cache quand une playlist est modifiée

5. **Modèle de données Music**:
   - La clé primaire composite de Music est (title, artist, album)
   - `release_date` est au format ISO 8601 (YYYY-MM-DD)
   - `genre` peut être null
