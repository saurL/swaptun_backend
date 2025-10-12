# Frontend Implementation Guide - FCM Notifications avec Données Imbriquées

## Vue d'ensemble

Le backend envoie désormais les notifications FCM avec des données JSON structurées et imbriquées au lieu d'un simple dictionnaire plat de strings.

## Structure des Données de Notification

### Format des Données

Les notifications FCM contiennent maintenant un champ `data` avec la structure suivante :

```json
{
  "type": "playlist_shared",
  "shared_notification": {
    "playlist_id": 123,
    "playlist_name": "My Awesome Playlist",
    "shared_by_id": 456,
    "shared_by_username": "john_doe",
    "shared_by_name": "John Doe",
    "route": "/home/shared"
  }
}
```

### Types de Notifications

#### 1. Playlist Partagée (`playlist_shared`)

**Champs disponibles dans `shared_notification` :**
- `playlist_id` (number) : ID de la playlist partagée
- `playlist_name` (string) : Nom de la playlist
- `shared_by_id` (number) : ID de l'utilisateur qui a partagé
- `shared_by_username` (string) : Nom d'utilisateur de celui qui a partagé
- `shared_by_name` (string) : Nom complet de celui qui a partagé
- `route` (string) : Route de navigation suggérée (ex: "/home/shared")

## Implémentation Frontend

### React Native / TypeScript

#### Types TypeScript

```typescript
// types/notification.ts
export interface NotificationData {
  type: string;
  shared_notification?: SharedNotificationData;
}

export interface SharedNotificationData {
  playlist_id: number;
  playlist_name: string;
  shared_by_id: number;
  shared_by_username: string;
  shared_by_name: string;
  route: string;
}
```

#### Parsing des Notifications

```typescript
import messaging from '@react-native-firebase/messaging';
import { NotificationData, SharedNotificationData } from './types/notification';

// Handler pour les notifications en background
messaging().setBackgroundMessageHandler(async (remoteMessage) => {
  console.log('Message handled in the background!', remoteMessage);

  if (remoteMessage.data) {
    handleNotificationData(remoteMessage.data);
  }
});

// Handler pour les notifications en foreground
useEffect(() => {
  const unsubscribe = messaging().onMessage(async (remoteMessage) => {
    console.log('Message handled in the foreground!', remoteMessage);

    if (remoteMessage.data) {
      handleNotificationData(remoteMessage.data);
    }
  });

  return unsubscribe;
}, []);

function handleNotificationData(data: any) {
  const notificationData: NotificationData = {
    type: data.type,
  };

  // Parse shared_notification si présent
  if (data.shared_notification) {
    try {
      // FCM envoie tout en string, donc on doit parser le JSON
      const sharedNotif: SharedNotificationData = typeof data.shared_notification === 'string'
        ? JSON.parse(data.shared_notification)
        : data.shared_notification;

      notificationData.shared_notification = sharedNotif;
    } catch (error) {
      console.error('Error parsing shared_notification:', error);
    }
  }

  // Router based on type
  switch (notificationData.type) {
    case 'playlist_shared':
      handlePlaylistShared(notificationData.shared_notification!);
      break;
    default:
      console.log('Unknown notification type:', notificationData.type);
  }
}

function handlePlaylistShared(data: SharedNotificationData) {
  console.log(`${data.shared_by_name} shared playlist: ${data.playlist_name}`);

  // Navigate to the shared playlist
  navigation.navigate(data.route, {
    playlistId: data.playlist_id,
  });

  // Or show a local notification
  showLocalNotification({
    title: 'New Shared Playlist',
    body: `${data.shared_by_name} shared "${data.playlist_name}" with you`,
    data: {
      playlistId: data.playlist_id,
      route: data.route,
    },
  });
}
```

### Flutter / Dart

#### Modèles de Données

```dart
// models/notification_data.dart
class NotificationData {
  final String type;
  final SharedNotificationData? sharedNotification;

  NotificationData({
    required this.type,
    this.sharedNotification,
  });

  factory NotificationData.fromMap(Map<String, dynamic> map) {
    return NotificationData(
      type: map['type'] as String,
      sharedNotification: map['shared_notification'] != null
          ? SharedNotificationData.fromJson(map['shared_notification'])
          : null,
    );
  }
}

class SharedNotificationData {
  final int playlistId;
  final String playlistName;
  final int sharedById;
  final String sharedByUsername;
  final String sharedByName;
  final String route;

  SharedNotificationData({
    required this.playlistId,
    required this.playlistName,
    required this.sharedById,
    required this.sharedByUsername,
    required this.sharedByName,
    required this.route,
  });

  factory SharedNotificationData.fromJson(dynamic json) {
    // Handle both string (JSON encoded) and Map
    final Map<String, dynamic> data = json is String
        ? jsonDecode(json)
        : json as Map<String, dynamic>;

    return SharedNotificationData(
      playlistId: int.parse(data['playlist_id'].toString()),
      playlistName: data['playlist_name'] as String,
      sharedById: int.parse(data['shared_by_id'].toString()),
      sharedByUsername: data['shared_by_username'] as String,
      sharedByName: data['shared_by_name'] as String,
      route: data['route'] as String,
    );
  }
}
```

#### Handler de Notifications

```dart
import 'package:firebase_messaging/firebase_messaging.dart';
import 'dart:convert';

class NotificationService {
  final FirebaseMessaging _fcm = FirebaseMessaging.instance;

  Future<void> initialize() async {
    // Request permission
    await _fcm.requestPermission();

    // Handle foreground messages
    FirebaseMessaging.onMessage.listen((RemoteMessage message) {
      print('Got a message in the foreground!');
      if (message.data.isNotEmpty) {
        _handleNotificationData(message.data);
      }
    });

    // Handle background messages
    FirebaseMessaging.onBackgroundMessage(_firebaseMessagingBackgroundHandler);

    // Handle notification tap when app is in background
    FirebaseMessaging.onMessageOpenedApp.listen((RemoteMessage message) {
      print('Notification tapped!');
      if (message.data.isNotEmpty) {
        _handleNotificationData(message.data);
      }
    });
  }

  void _handleNotificationData(Map<String, dynamic> data) {
    try {
      final notificationData = NotificationData.fromMap(data);

      switch (notificationData.type) {
        case 'playlist_shared':
          _handlePlaylistShared(notificationData.sharedNotification!);
          break;
        default:
          print('Unknown notification type: ${notificationData.type}');
      }
    } catch (e) {
      print('Error handling notification: $e');
    }
  }

  void _handlePlaylistShared(SharedNotificationData data) {
    print('${data.sharedByName} shared playlist: ${data.playlistName}');

    // Navigate to the route
    navigatorKey.currentState?.pushNamed(
      data.route,
      arguments: {'playlistId': data.playlistId},
    );
  }
}

// Top-level function for background handler
Future<void> _firebaseMessagingBackgroundHandler(RemoteMessage message) async {
  print('Handling a background message: ${message.messageId}');
  // Process notification data
}
```

### Tauri (Rust)

#### Structures de Données

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationData {
    #[serde(rename = "type")]
    pub notification_type: String,
    pub shared_notification: Option<SharedNotificationData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SharedNotificationData {
    pub playlist_id: i32,
    pub playlist_name: String,
    pub shared_by_id: i32,
    pub shared_by_username: String,
    pub shared_by_name: String,
    pub route: String,
}
```

#### Handler de Notifications

```rust
use tauri_plugin_push_notifications::PushNotificationMessage;

fn handle_notification(message: PushNotificationMessage) {
    if let Some(data) = message.data {
        // Parse notification data
        match serde_json::from_value::<NotificationData>(data) {
            Ok(notification_data) => {
                match notification_data.notification_type.as_str() {
                    "playlist_shared" => {
                        if let Some(shared_notif) = notification_data.shared_notification {
                            handle_playlist_shared(shared_notif);
                        }
                    }
                    _ => println!("Unknown notification type: {}", notification_data.notification_type),
                }
            }
            Err(e) => eprintln!("Failed to parse notification data: {:?}", e),
        }
    }
}

fn handle_playlist_shared(data: SharedNotificationData) {
    println!("{} shared playlist: {}", data.shared_by_name, data.playlist_name);

    // Navigate or show UI
    // app_handle.emit_all("navigate", data.route).unwrap();
}
```

## Points Importants

### 1. Parsing FCM

⚠️ **Attention** : FCM envoie toutes les valeurs du champ `data` en tant que **strings**, même les objets JSON imbriqués. Vous devez :

1. Vérifier si la valeur est une string
2. Parser la string JSON si nécessaire
3. Convertir les types appropriés (ex: "123" → 123 pour les IDs)

### 2. Validation

Toujours valider les données reçues :

```typescript
function isValidSharedNotification(data: any): data is SharedNotificationData {
  return (
    typeof data === 'object' &&
    typeof data.playlist_id === 'number' &&
    typeof data.playlist_name === 'string' &&
    typeof data.shared_by_id === 'number' &&
    typeof data.shared_by_username === 'string' &&
    typeof data.shared_by_name === 'string' &&
    typeof data.route === 'string'
  );
}
```

### 3. Gestion des Erreurs

Toujours entourer le parsing avec un try-catch :

```typescript
try {
  const sharedNotif = JSON.parse(data.shared_notification);
  // Use sharedNotif
} catch (error) {
  console.error('Invalid notification data', error);
  // Fallback behavior
}
```

## Testing

### Test Payload pour Postman

```json
{
  "user_id": 1,
  "title": "New Shared Playlist",
  "body": "John Doe shared 'Rock Classics' with you",
  "data": {
    "type": "playlist_shared",
    "shared_notification": {
      "playlist_id": 123,
      "playlist_name": "Rock Classics",
      "shared_by_id": 456,
      "shared_by_username": "john_doe",
      "shared_by_name": "John Doe",
      "route": "/home/shared"
    }
  }
}
```

## Migration depuis l'Ancien Format

Si vous utilisiez auparavant un format plat, voici comment migrer :

### Ancien Format (dépréci é)
```json
{
  "type": "playlist_shared",
  "playlist_id": "123",
  "playlist_name": "Rock",
  "shared_by_id": "456",
  // ...
}
```

### Nouveau Format
```json
{
  "type": "playlist_shared",
  "shared_notification": {
    "playlist_id": 123,
    "playlist_name": "Rock",
    "shared_by_id": 456,
    // ...
  }
}
```

### Code de Compatibilité

```typescript
function parseNotificationData(data: any): NotificationData {
  // New format (preferred)
  if (data.shared_notification) {
    return {
      type: data.type,
      shared_notification: typeof data.shared_notification === 'string'
        ? JSON.parse(data.shared_notification)
        : data.shared_notification,
    };
  }

  // Legacy format (fallback)
  if (data.type === 'playlist_shared' && data.playlist_id) {
    return {
      type: data.type,
      shared_notification: {
        playlist_id: parseInt(data.playlist_id),
        playlist_name: data.playlist_name,
        shared_by_id: parseInt(data.shared_by_id),
        shared_by_username: data.shared_by_username,
        shared_by_name: data.shared_by_name,
        route: data.route || '/home/shared',
      },
    };
  }

  throw new Error('Invalid notification format');
}
```

## Support

Pour toute question ou problème, référez-vous au backend ou ouvrez une issue.
