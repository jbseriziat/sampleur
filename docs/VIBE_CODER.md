# Sampleur V2 — Guide Vibe Coder

> Ce guide est fait pour toi si tu travailles avec Claude Code (ou un autre IA) pour faire évoluer l'application sans forcément maîtriser Rust ou TypeScript à fond. L'objectif : comprendre comment l'appli est construite, où se trouvent les fichiers importants, et comment demander des évolutions à ton IA de façon efficace.

---

## C'est quoi Sampleur V2 ?

Sampleur V2 est une **application de bureau** (Windows / Linux / macOS) qui fonctionne comme un **sampler MIDI** : tu charges des sons sur des pads (comme un Launchpad Novation), tu les déclenches au clavier MIDI ou à la souris, et tu peux appliquer des effets audio dessus.

Concrètement :
- **64 pads** disposés en grille 8×8 (ou 4×4)
- Chaque pad peut charger un fichier audio (WAV, MP3, FLAC, OGG)
- 3 modes de jeu : **oneshot** (joue une fois), **loop** (boucle), **hold** (joue tant que c'est enfoncé)
- **Effets globaux** : distortion, filtre, delay, reverb, gate, flanger
- **Support MIDI** : n'importe quelle interface MIDI, support étendu du **Novation Launchpad MK2** (LEDs colorées)
- **Presets** : sauvegarde/chargement de kits complets en `.sampleur2`
- **Enregistrement live** en WAV lossless
- **Drag & drop** pour réorganiser les pads visuellement

---

## L'architecture en 2 minutes

Sampleur V2 est une app **Tauri** : c'est comme une page web qui peut accéder aux ressources de ton ordinateur.

```
┌────────────────────────────────────────┐
│  CE QUE TU VOIS (React = JavaScript)  │
│  - Grille de pads                      │
│  - Boutons, sliders, menus             │
│  - Affiché dans une fenêtre WebKit     │
└───────────────┬────────────────────────┘
                │ invoke() → appelle des fonctions Rust
                │ ← emit() Rust envoie des événements à JS
┌───────────────▼────────────────────────┐
│  LE MOTEUR (Rust = code bas niveau)   │
│  - Lit et joue les fichiers audio      │
│  - Applique les effets en temps réel   │
│  - Gère les entrées/sorties MIDI       │
│  - Lit/écrit les presets sur le disque │
└────────────────────────────────────────┘
```

**Règle simple :** tout ce qui touche à l'audio, au MIDI, aux fichiers → c'est dans Rust (dossier `src-tauri/`). Tout ce qui touche à l'interface visuelle → c'est dans React (dossier `src/`).

---

## Carte des fichiers importants

### Côté interface (dossier `sampleur-v2/src/`)

| Fichier | Rôle simple |
|---------|------------|
| `App.tsx` | La racine : assemble tous les panneaux |
| `components/Header.tsx` | Barre du haut : nom du kit, BPM, bouton enregistrement |
| `components/PadGrid.tsx` | La grille de pads, gère le drag & drop |
| `components/Pad.tsx` | Un pad individuel (couleur, mode, barre de progression) |
| `components/PadEditor.tsx` | Panneau d'édition d'un pad (son, mode, volume, détune…) |
| `components/FxPanel.tsx` | Les sliders d'effets audio |
| `components/PresetPanel.tsx` | Sauvegarde/chargement de kits, configuration MIDI |
| `store/usePadStore.ts` | **La mémoire des 64 pads** (état centralisé) |
| `store/useFxStore.ts` | **La mémoire des effets et du BPM** |
| `store/useMidiStore.ts` | État des appareils MIDI connectés |
| `hooks/useTauriEvents.ts` | Reçoit les événements venant de Rust (progression, MIDI…) |
| `types/index.ts` | Les structures de données (PadState, FxState, ColorDef…) |

### Côté moteur (dossier `sampleur-v2/src-tauri/src/`)

| Fichier | Rôle simple |
|---------|------------|
| `state.rs` | Toutes les commandes audio et l'état partagé |
| `audio/engine.rs` | Le cœur du moteur audio (mixage, effets, enregistrement) |
| `audio/pad.rs` | Comment un pad joue son son (vitesse, boucle, hold) |
| `audio/loader.rs` | Ouvre et décode les fichiers audio |
| `audio/resampler.rs` | Convertit tous les sons à 48 000 Hz |
| `audio/effects/mod.rs` | La chaîne d'effets complète |
| `midi/engine.rs` | Reçoit les notes MIDI, les route vers les pads |
| `midi/launchpad.rs` | Contrôle les LEDs du Launchpad MK2 |
| `preset/schema.rs` | La structure d'un preset (.sampleur2) |
| `preset/io.rs` | Sauvegarde et chargement des presets |
| `commands/audio_commands.rs` | Fonctions appelées par l'interface (charger son, déclencher pad…) |
| `commands/fx_commands.rs` | Fonctions pour changer les effets |
| `commands/midi_commands.rs` | Fonctions pour la configuration MIDI |

---

## Comment le frontend parle au backend

Quand tu cliques sur "Charger un son" dans l'interface, voici ce qui se passe :

```
1. Le composant React appelle :
   invoke("load_sample", { padId: 3, filePath: "/home/moi/kick.wav" })

2. Rust reçoit l'appel dans :
   commands/audio_commands.rs → fn load_sample(pad_id: usize, file_path: String)

3. Rust décode le fichier, rééchantillonne à 48 kHz, envoie le son au moteur audio

4. Rust émet un événement vers React :
   app_handle.emit("sample-loaded", { padId: 3, fileName: "kick.wav", durationSecs: 0.5 })

5. React reçoit l'événement dans :
   hooks/useTauriEvents.ts → listen("sample-loaded", ...)
   → met à jour le store Zustand (usePadStore)
   → l'interface se rafraîchit automatiquement
```

---

## Les "stores" Zustand : la mémoire de l'interface

L'interface utilise **Zustand** pour mémoriser l'état. Pense à ça comme des variables globales propres.

**`usePadStore`** — l'état des 64 pads :
```
pads[0..63] :
  - label      (nom affiché)
  - color      (couleur + info Launchpad)
  - mode       ("oneshot" | "loop" | "hold")
  - hasSample  (est-ce qu'un son est chargé ?)
  - volume     (0 à 2)
  - detuneCents (hauteur, -1200 à +1200)
  - originalBpm (BPM d'origine du sample)
  - midiNote   (note MIDI assignée)
  - isPlaying  (est-ce que le pad joue en ce moment ?)
  - progress   (0 à 1 pour la barre de progression)
  - filePath   (chemin du fichier sur le disque)
```

**`useFxStore`** — les effets :
```
fx :
  - distortion, filterFreq, filterResonance
  - delayTime, delayFeedback, delayMix
  - reverbMix, gateRate
  - flangerDepth, flangerRate
  - masterVolume
bpm       (tempo global)
quantize  (déclencher sur le temps ?)
```

---

## Tâches courantes avec Claude Code

### ✅ "Ajouter un nouveau slider d'effet"

1. Ajouter le paramètre dans `src/types/index.ts` (interface `FxState`)
2. Ajouter la valeur par défaut dans `src/store/useFxStore.ts`
3. Ajouter le slider dans `src/components/FxPanel.tsx`
4. Ajouter le variant dans `state.rs` (enum `FxParam`)
5. Gérer le variant dans `audio/engine.rs` (`handle_command`)
6. Ajouter le cas dans `commands/fx_commands.rs`
7. Ajouter la logique audio dans `audio/effects/mod.rs`

### ✅ "Changer le comportement d'un pad"

Aller dans `src-tauri/src/audio/pad.rs` → struct `PadPlayer` et sa méthode `render()`.

### ✅ "Ajouter un bouton dans l'interface"

Travailler dans le composant React concerné (`Header.tsx`, `PadEditor.tsx`, etc.) et utiliser `invoke()` pour appeler une fonction Rust existante.

### ✅ "Changer la couleur ou le style d'un composant"

Tailwind CSS est utilisé. Modifier directement les classes dans les fichiers `.tsx`. Voir la doc Tailwind : https://tailwindcss.com/docs

### ✅ "Modifier le format de preset"

1. Modifier `preset/schema.rs` (structs Rust)
2. Modifier `src/types/index.ts` (interfaces TypeScript)
3. Mettre à jour la logique dans `preset/io.rs`
4. Mettre à jour `PresetPanel.tsx` si l'UI change

### ✅ "Ajouter un nouveau mode de jeu pour les pads"

1. Ajouter le variant dans l'enum `PadMode` dans `state.rs` (Rust) et `types/index.ts` (TS)
2. Gérer le nouveau mode dans `audio/pad.rs` → méthode `render()`
3. Ajouter l'icône/label dans `Pad.tsx`
4. Ajouter l'option dans `PadEditor.tsx`

---

## Commandes utiles

```bash
# Lancer en mode développement (avec rechargement automatique)
cd sampleur-v2
npm run tauri dev

# Vérifier que TypeScript est correct (sans compiler)
npx tsc --noEmit

# Vérifier que Rust compile (sans tout builder)
cargo build --manifest-path src-tauri/Cargo.toml

# Build de production complet (.deb + .rpm + AppImage)
npm run tauri build
```

---

## Conseils pour bien travailler avec Claude Code

### Contexte de session

Au début d'une nouvelle session Claude Code, fournis ce contexte :

```
Je travaille sur Sampleur V2, une app Tauri (Rust + React/TypeScript).
Répertoire : /home/jb/Dev/Sampleur-Project/sampleur-v2/
- Frontend React : src/
- Backend Rust : src-tauri/src/
- Docs : /home/jb/Dev/Sampleur-Project/docs/
Stack : Tauri v2.10, Rust 1.94, React 18, TypeScript 5.8, Tailwind v3, Zustand v5
```

### Formulations efficaces

| Plutôt que... | Dis plutôt... |
|---------------|--------------|
| "Fais quelque chose avec le volume" | "Ajoute un slider de volume master dans Header.tsx qui appelle la commande Rust `set_fx_param('masterVolume', value)`" |
| "Améliore le pad" | "Dans `Pad.tsx`, ajoute l'affichage de la durée du sample (en secondes) sous le nom du fichier" |
| "Corrige le MIDI" | "Dans `midi/engine.rs`, le bouton Stop All (CC 64) n'est pas géré — ajoute un handler pour envoyer `AudioCommand::StopAll`" |

### Demander une exploration avant de coder

Dis toujours à Claude de **lire les fichiers concernés avant de modifier** :

> "Avant de modifier, lis `src-tauri/src/audio/pad.rs` et `src/components/PadEditor.tsx` pour comprendre le contexte."

### Quand quelque chose ne fonctionne pas

Demande à Claude de **lancer le TypeScript check et le build Rust** après chaque modification :

> "Après avoir fait les modifications, vérifie avec `npx tsc --noEmit` et `cargo build --manifest-path src-tauri/Cargo.toml`."

---

## Fichier CLAUDE.md (à placer dans sampleur-v2/)

Ce fichier est lu automatiquement par Claude Code au démarrage d'une session dans ce dossier. Il fournit le contexte du projet.

> ⚠️ Ce fichier est déjà créé dans le projet (voir `sampleur-v2/CLAUDE.md`).

Son contenu résume : stack, structure des dossiers, commandes de build, règles de code, points d'attention (AppImage FUSE, HTML5 DnD, etc.).

---

## Points d'attention importants

### ⚠️ Le drag & drop utilise des mouse events, pas HTML5 DnD

HTML5 DnD ne fonctionne pas dans Tauri/WebKit. On utilise `mousedown`/`mouseenter`/`mouseup` + des `useRef` pour éviter les bugs de closure. **Ne pas revenir à HTML5 DnD.**

### ⚠️ Les pads s'échangent mais les notes MIDI restent sur la position

`swapPads()` dans `usePadStore.ts` échange tout SAUF `id` et `midiNote`. C'est voulu : la note MIDI est liée à la position physique sur le Launchpad, pas au contenu du pad.

### ⚠️ Les sons sont tous convertis à 48 000 Hz au chargement

Ne pas changer `TARGET_SAMPLE_RATE = 48000` dans `commands/audio_commands.rs` sans mettre à jour la création du WAV dans `start_recording`.

### ⚠️ Les effets s'appliquent GLOBALEMENT

La chaîne d'effets est partagée par tous les pads. Il n'y a pas d'effets par pad (pour l'instant).

### ⚠️ Rust est strict sur les types

Si tu ajoutes un champ dans `PadConfig` (schema.rs), il faut aussi le gérer dans `preset/io.rs` et dans `commands/audio_commands.rs`. Rust ne compilera pas si quelque chose est oublié.

---

## Structure d'un preset `.sampleur2`

C'est juste un fichier JSON renommé. Tu peux l'ouvrir avec un éditeur de texte :

```json
{
  "version": 2,
  "name": "Mon Kit",
  "bpm": 120.0,
  "quantize": false,
  "gridSize": 64,
  "kitMode": "lightweight",
  "fx": {
    "distortion": 0,
    "filterFreq": 20000,
    ...
  },
  "pads": [
    {
      "id": 0,
      "label": "Kick",
      "mode": "oneshot",
      "volume": 1.0,
      "detuneCents": 0,
      "sample": {
        "fileName": "kick.wav",
        "absolutePathHint": "/home/jb/Samples/kick.wav"
      }
    },
    null,
    null,
    ...
  ]
}
```

---

## Glossaire

| Terme | Signification |
|-------|--------------|
| **Tauri** | Framework pour faire des apps desktop avec une UI web (ici React) et un backend Rust |
| **Invoke** | Appel du frontend vers une fonction Rust |
| **Emit** | Message envoyé par Rust vers le frontend |
| **Zustand** | Bibliothèque de gestion d'état pour React (comme Redux mais simple) |
| **CPAL** | Bibliothèque Rust d'accès à la carte son (cross-platform) |
| **Symphonia** | Décodeur audio Rust (lit WAV, MP3, FLAC, OGG, AAC) |
| **rubato** | Bibliothèque Rust de rééchantillonnage audio |
| **midir** | Bibliothèque Rust pour le MIDI |
| **hound** | Bibliothèque Rust pour écrire des fichiers WAV |
| **Freeverb** | Algorithme de réverb classique (Schroeder) |
| **mpsc** | Canal de messages Rust entre threads (comme une file d'attente) |
| **SysEx** | Messages MIDI spéciaux pour configurer un appareil (ici : LEDs du Launchpad) |
| **AppImage** | Format d'application Linux portable (un seul fichier exécutable) |
| **Quantize** | Déclencher les sons exactement sur le temps (synchronisé au BPM) |

---

*Sampleur V2 v2.0.0 — Guide vibe coder — 2026-03-29*
