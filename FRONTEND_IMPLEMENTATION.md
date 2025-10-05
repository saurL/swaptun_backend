# Consignes d'implémentation Frontend - Export de Playlist vers Apple Music

## Vue d'ensemble

Une nouvelle fonctionnalité a été ajoutée au backend permettant d'exporter une playlist de notre base de données vers Apple Music. Ce document décrit comment implémenter cette fonctionnalité côté frontend.

## Endpoint API

### Export d'une playlist vers Apple Music

**Endpoint:** `POST /api/playlists/{id}/send`

**Headers requis:**
- `Authorization: Bearer <jwt_token>`

**Body de la requête:**
```json
{
  "destination": "AppleMusic"
}
```

**Paramètres:**
- `{id}`: ID de la playlist à exporter (integer)

**Réponse en cas de succès (200 OK):**
```json
"Playlist sent to Apple Music successfully with ID: <apple_playlist_id>"
```

**Erreurs possibles:**
- `401 Unauthorized`: Token JWT invalide ou manquant
- `404 Not Found`: Playlist non trouvée
- `500 Internal Server Error`: Erreur lors de la création de la playlist sur Apple Music ou token Apple Music manquant

## Cas d'usage

### 1. Prérequis

Avant d'exporter une playlist vers Apple Music, l'utilisateur doit :
1. Être connecté à l'application (avoir un token JWT valide)
2. Avoir connecté son compte Apple Music (avoir un token Apple Music valide en base de données)

### 2. Flux d'export d'une playlist

1. **Interface utilisateur**
   - Afficher un bouton "Exporter vers Apple Music" sur la page de détails d'une playlist
   - Le bouton doit être désactivé si l'utilisateur n'a pas connecté son compte Apple Music

2. **Action de l'utilisateur**
   - L'utilisateur clique sur le bouton "Exporter vers Apple Music"
   - Afficher un loader/spinner pendant le traitement

3. **Appel API**
   ```javascript
   const exportToAppleMusic = async (playlistId) => {
     try {
       const response = await fetch(`/api/playlists/${playlistId}/send`, {
         method: 'POST',
         headers: {
           'Authorization': `Bearer ${jwtToken}`,
           'Content-Type': 'application/json'
         },
         body: JSON.stringify({
           destination: 'AppleMusic'
         })
       });

       if (!response.ok) {
         throw new Error('Failed to export playlist');
       }

       const message = await response.text();
       // Afficher un message de succès
       console.log(message);
     } catch (error) {
       // Gérer l'erreur
       console.error('Error exporting playlist:', error);
     }
   };
   ```

4. **Réponse de l'API**
   - En cas de succès : Afficher un message de confirmation avec l'ID de la playlist créée sur Apple Music
   - En cas d'erreur : Afficher un message d'erreur approprié

### 3. Comportement de l'export

L'export effectue les actions suivantes côté backend :

1. **Création de la playlist sur Apple Music**
   - Une nouvelle playlist est créée sur Apple Music avec le même nom et description que la playlist source
   - Retourne l'ID de la playlist Apple Music créée

2. **Recherche et ajout des morceaux**
   - Pour chaque morceau de la playlist :
     - Recherche le morceau sur Apple Music par titre et artiste
     - Compare les résultats pour trouver une correspondance exacte (titre et artiste)
     - Ajoute d'abord le morceau à la bibliothèque Apple Music de l'utilisateur
     - Puis ajoute le morceau à la playlist créée

3. **Gestion des morceaux non trouvés**
   - Les morceaux qui ne sont pas trouvés sur Apple Music sont simplement ignorés
   - Ils sont loggés côté backend mais n'empêchent pas la création de la playlist
   - L'export continue même si certains morceaux ne sont pas trouvés

### 4. Limitations et considérations

- **Morceaux par batch** : Les morceaux sont ajoutés par lots de 100 maximum pour respecter les limites de l'API Apple Music
- **Correspondance exacte** : Seuls les morceaux avec titre ET artiste correspondant exactement sont ajoutés
- **Token Apple Music requis** : L'utilisateur doit avoir un token Apple Music valide (obtenu via MusicKit JS côté frontend)

## Autres endpoints liés à Apple Music

### Connexion Apple Music

**Endpoint:** `POST /api/apple/token`

**Body:**
```json
{
  "token": "<apple_music_user_token>"
}
```

**Usage:** Sauvegarder le token utilisateur Apple Music obtenu via MusicKit JS

### Obtenir le Developer Token

**Endpoint:** `GET /api/apple/developer-token`

**Réponse:**
```json
{
  "developer_token": "<apple_developer_token>"
}
```

**Usage:** Récupérer le developer token pour initialiser MusicKit JS côté frontend

### Synchroniser les playlists depuis Apple Music

**Endpoint:** `POST /api/apple/synchronize`

**Usage:** Importer les playlists Apple Music de l'utilisateur dans notre base de données

### Déconnexion Apple Music

**Endpoint:** `DELETE /api/apple/disconnect`

**Usage:** Supprimer le token Apple Music et toutes les playlists importées depuis Apple Music

## Exemple d'implémentation complète

```javascript
// Composant React exemple
import { useState } from 'react';

const PlaylistExport = ({ playlist, userHasAppleMusic }) => {
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState(null);

  const handleExportToAppleMusic = async () => {
    setIsExporting(true);
    setError(null);

    try {
      const response = await fetch(`/api/playlists/${playlist.id}/send`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('jwt')}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          destination: 'AppleMusic'
        })
      });

      if (!response.ok) {
        throw new Error('Export failed');
      }

      const message = await response.text();
      alert(`Succès: ${message}`);
    } catch (err) {
      setError('Impossible d\'exporter la playlist vers Apple Music');
      console.error(err);
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div>
      <button
        onClick={handleExportToAppleMusic}
        disabled={!userHasAppleMusic || isExporting}
      >
        {isExporting ? 'Export en cours...' : 'Exporter vers Apple Music'}
      </button>
      {error && <p style={{ color: 'red' }}>{error}</p>}
      {!userHasAppleMusic && (
        <p>Vous devez d'abord connecter votre compte Apple Music</p>
      )}
    </div>
  );
};
```

## Notes importantes

1. **Migration base de données** : Une nouvelle migration a été créée pour ajouter "AppleMusic" à l'enum `playlist_origin` si elle n'existe pas déjà

2. **Même endpoint pour toutes les plateformes** : L'endpoint `/api/playlists/{id}/send` est utilisé pour exporter vers toutes les plateformes (Spotify, YouTube Music, Apple Music, Deezer). Seul le champ `destination` change dans le body

3. **Valeurs possibles pour destination** :
   - `"Spotify"`
   - `"YoutubeMusic"`
   - `"AppleMusic"`
   - `"Deezer"` (pas encore implémenté)

4. **Tests** : Des tests unitaires ont été ajoutés côté backend pour valider la logique d'export
