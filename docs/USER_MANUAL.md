# Sampleur V2 — Manuel d'utilisation

**Version 2.0.0** — Application de sampling MIDI pour Linux, Windows et macOS

---

## Table des matières

1. [Introduction](#1-introduction)
2. [Installation](#2-installation)
3. [Premier lancement](#3-premier-lancement)
4. [Présentation de l'interface](#4-présentation-de-linterface)
5. [Travailler avec les pads](#5-travailler-avec-les-pads)
6. [Modes de déclenchement](#6-modes-de-déclenchement)
7. [Le panneau d'édition d'un pad](#7-le-panneau-dédition-dun-pad)
8. [Les effets audio (FX)](#8-les-effets-audio-fx)
9. [Les presets — Sauvegarder et charger](#9-les-presets--sauvegarder-et-charger)
10. [Configuration MIDI](#10-configuration-midi)
11. [Novation Launchpad MK2](#11-novation-launchpad-mk2)
12. [Enregistrement live](#12-enregistrement-live)
13. [Réorganiser les pads par glisser-déposer](#13-réorganiser-les-pads-par-glisser-déposer)
14. [Astuces et bonnes pratiques](#14-astuces-et-bonnes-pratiques)
15. [Dépannage](#15-dépannage)

---

## 1. Introduction

**Sampleur V2** est un sampler multi-pads qui te permet de :
- Charger des sons (WAV, MP3, FLAC, OGG) sur jusqu'à **64 pads**
- Les déclencher à la souris ou via un **contrôleur MIDI**
- Appliquer une **chaîne d'effets** (distortion, filtre, delay, reverb, gate, flanger)
- Sauvegarder et partager tes **kits** en fichiers `.sampleur2`
- **Enregistrer** ton mix en qualité lossless (WAV 32-bit)
- Utiliser un **Novation Launchpad MK2** avec retour visuel LED

---

## 2. Installation

### Linux (recommandé : AppImage)

1. Télécharge le fichier `Sampleur_2.0.0_amd64.AppImage`
2. Rends-le exécutable :
   - Clic droit → Propriétés → Autoriser l'exécution comme programme
   - Ou dans un terminal : `chmod +x Sampleur_2.0.0_amd64.AppImage`
3. Double-clique dessus — l'application démarre sans installation

### Linux (paquets système)

- **Ubuntu/Debian :** `sudo dpkg -i Sampleur_2.0.0_amd64.deb`
- **Fedora/openSUSE :** `sudo rpm -i Sampleur-2.0.0-1.x86_64.rpm`

### Première utilisation avec un Launchpad MK2

Branche le Launchpad **avant** de lancer Sampleur pour une meilleure détection automatique.

---

## 3. Premier lancement

À l'ouverture, Sampleur affiche une grille de **64 pads vides** en mode 8×8.

**Pour commencer rapidement :**

1. Clique sur le bouton **CONFIG** en haut à droite pour entrer en mode édition
2. Clique sur un pad pour le sélectionner (il s'entoure d'un halo)
3. Dans le panneau de droite, clique sur **📂 Charger un son**
4. Sélectionne un fichier audio (WAV, MP3, FLAC, OGG)
5. Clique sur le pad pour déclencher le son !

---

## 4. Présentation de l'interface

```
┌──────────────────────────────────────────────────────────────────┐
│  NOM DU KIT  [4x4|8x8] [BPM: 120] [⏱ Quantize] [■ Stop All]   │
│              [CONFIG] [JEDI]  [● REC]                            │
├──────────┬───────────────────────────────────┬────────────────────┤
│          │                                   │                    │
│  FX      │   GRILLE DE PADS (8×8)            │   ÉDITEUR DE PAD   │
│  PANEL   │                                   │   (si un pad est   │
│  (si     │   [ ][ ][ ][ ][ ][ ][ ][ ]        │    sélectionné)    │
│  JEDI    │   [ ][ ][ ][ ][ ][ ][ ][ ]        │                    │
│  actif)  │   [ ][ ][ ][ ][ ][ ][ ][ ]        │                    │
│          │   ...                             │                    │
├──────────┴───────────────────────────────────┴────────────────────┤
│  PRESETS: [💾 Sauvegarder] [📂 Charger]  │  MIDI: [Input ▼][Output ▼] [↺]  │
└──────────────────────────────────────────────────────────────────┘
```

### La barre du haut (Header)

| Élément | Description |
|---------|-------------|
| **Nom du kit** | Clique dessus pour renommer ton kit |
| **4×4 / 8×8** | Bascule entre 16 et 64 pads |
| **BPM** | Tempo global (affecte les pads avec un BPM original défini) |
| **⏱ Quantize** | Active le déclenchement quantifié sur le temps |
| **■ Stop All** | Arrête tous les sons en cours |
| **Nouveau kit** | Remet tous les pads à zéro |
| **CONFIG** | Active/désactive le mode édition (sélection et configuration des pads) |
| **JEDI** | Affiche/masque le panneau d'effets FX sur la gauche |
| **● REC / ■ STOP REC** | Lance/arrête l'enregistrement live |

### La grille de pads

- En **mode normal** : cliquer sur un pad le déclenche
- En **mode CONFIG** : cliquer sur un pad le sélectionne pour l'éditer

### Le panneau de bas (Presets + MIDI)

Contient les commandes de sauvegarde/chargement et la sélection des appareils MIDI.

---

## 5. Travailler avec les pads

### Déclencher un son

- **Clic gauche** sur un pad → déclenche le son (selon le mode)
- **Touche MIDI** assignée → déclenche le son depuis ton contrôleur

### Comprendre l'affichage d'un pad

Chaque pad affiche :
- Sa **couleur** (personnalisable par pad)
- Son **icône de mode** :
  - `▷` = oneshot (joue une fois)
  - `∞` = loop (boucle)
  - `⊙` = hold (joue tant que c'est maintenu)
- Le **nom du fichier** chargé (tronqué si trop long)
- Une **barre de progression** en bas pendant la lecture

### Pad en cours de lecture

Un pad qui joue s'illumine plus fortement. La barre de progression à sa base avance en temps réel.

---

## 6. Modes de déclenchement

Chaque pad peut avoir un mode différent :

### ▷ Oneshot (joue une fois)
- Le son démarre et joue jusqu'à la fin
- Cliquer à nouveau redémarre depuis le début
- Idéal pour : percussions, effets sonores courts

### ∞ Loop (boucle)
- Le son joue en boucle continue
- Un second clic l'arrête
- Idéal pour : nappes, drones, boucles rythmiques

### ⊙ Hold (maintenu)
- Le son joue SEULEMENT tant que tu maintiens le clic (ou la touche MIDI)
- Relâcher stoppe le son
- Idéal pour : one-shots longs, ambiances à déclencher manuellement

---

## 7. Le panneau d'édition d'un pad

Active le **mode CONFIG** puis clique sur un pad pour ouvrir son éditeur sur la droite.

### Chargement du son

- **📂 Charger** : Ouvre un explorateur de fichiers pour sélectionner un son
- **✕ Supprimer** : Retire le son du pad
- Formats supportés : **WAV, MP3, FLAC, OGG, AAC**

### Paramètres du pad

| Paramètre | Valeur | Description |
|-----------|--------|-------------|
| **Mode** | oneshot / loop / hold | Mode de déclenchement |
| **Volume** | 0 à 2 | Niveau du pad (1 = normal, 2 = +6 dB) |
| **Détune** | -1200 à +1200 cents | Hauteur du son (±1 octave). 0 = hauteur d'origine |
| **BPM original** | 60 à 200 | Si défini, le son est accéléré/ralenti pour suivre le BPM global |
| **Couleur** | 8 couleurs | Couleur visuelle + couleur de LED Launchpad |
| **Note MIDI** | 0-127 | Note MIDI qui déclenche ce pad |

### Assigner une note MIDI (MIDI Learn)

1. Clique sur **MIDI Learn** dans l'éditeur du pad
2. Appuie sur la touche de ton contrôleur MIDI
3. La note est automatiquement assignée

---

## 8. Les effets audio (FX)

Clique sur **JEDI** pour afficher le panneau d'effets à gauche.

> ⚠️ Les effets s'appliquent à **tous les pads** simultanément (pas par pad individuellement).

La chaîne d'effets est traitée dans cet ordre :

### 1. Distortion
- **Valeur : 0 à 10**
- Ajoute de la saturation/crunch au signal
- 0 = aucune distortion, 10 = saturation maximale
- Type : saturation atan (douce et musicale)

### 2. Filtre (Filter)
- **Fréquence : 20 Hz à 20 000 Hz**
- **Résonance : 0.1 à 10**
- Filtre passe-bas : coupe les fréquences au-dessus de la valeur choisie
- Résonance haute = accent sur la fréquence de coupure (effet "wah")
- Par défaut : 20 000 Hz (ouvert = transparent)

### 3. Delay
- **Temps : 0 à 2 secondes**
- **Feedback : 0 à 0.95** (répétitions)
- **Mix : 0 à 1** (dosage de l'effet)
- Écho numérique. Mix à 0 = effet désactivé.

### 4. Reverb
- **Mix : 0 à 1**
- Réverbération (Freeverb, algorithme Schroeder)
- 0 = son sec, 1 = réverb maximum

### 5. Gate (LFO)
- **Taux : 0 à 20 Hz**
- Coupe le signal à intervalles réguliers (effet "sidechain" ou "trance gate")
- 0 = désactivé

### 6. Flanger
- **Profondeur : 0 à 0.02**
- **Taux : 0 à 5 Hz**
- Effet de chorus/flange (modulation de hauteur légère)
- Profondeur à 0 = désactivé

### 7. Volume master
- **0 à 2** (1 = niveau normal)
- Volume global de sortie

---

## 9. Les presets — Sauvegarder et charger

Un **preset** (ou kit) contient :
- La configuration de tous les pads (sons, modes, volumes, couleurs, notes MIDI)
- Les paramètres d'effets FX
- Le BPM et l'état du quantize
- La taille de la grille (4×4 ou 8×8)

Les fichiers de preset ont l'extension `.sampleur2`.

### Sauvegarder un preset

1. Dans le panneau du bas, clique sur **💾 Sauvegarder**
2. Choisis un emplacement et un nom
3. Sélectionne le **mode du kit** :
   - **Léger** : Le fichier est petit (~10 Ko). Les sons restent à leur emplacement d'origine. ⚠️ Si tu déplaces les sons, le preset ne les retrouvera plus.
   - **Portable** : Le fichier est accompagné d'un dossier `_samples/` contenant une copie de tous les sons. Idéal pour partager ou déplacer sur un autre PC.

### Charger un preset

1. Clique sur **📂 Charger** dans le panneau du bas
2. Sélectionne un fichier `.sampleur2`
3. L'application charge automatiquement tous les sons et restaure les paramètres

> **Astuce :** Tu peux aussi double-cliquer sur un fichier `.sampleur2` depuis ton gestionnaire de fichiers (si l'association de fichiers est configurée) pour l'ouvrir directement dans Sampleur.

### Renommer le kit

Clique sur le nom du kit en haut à gauche de l'interface pour le renommer.

---

## 10. Configuration MIDI

### Sélectionner les appareils

Dans le panneau de bas :
- **Input MIDI** : L'appareil qui envoie des notes vers Sampleur (Launchpad, clavier MIDI, etc.)
- **Output MIDI** : L'appareil qui reçoit des messages de Sampleur (pour les LEDs du Launchpad)

### Rafraîchir la liste des appareils

Clique sur le bouton **↺** à côté des sélecteurs MIDI pour rescanner les appareils branchés.

> Si tu as branché un appareil MIDI après avoir lancé Sampleur, utilise ce bouton.

### Assigner des notes MIDI aux pads

**Méthode 1 — MIDI Learn (recommandé)**
1. Active le mode **CONFIG**
2. Sélectionne un pad
3. Dans l'éditeur, clique sur **MIDI Learn**
4. Appuie sur la touche de ton contrôleur
5. La note est assignée

**Méthode 2 — Valeur manuelle**
Dans l'éditeur de pad, entre directement le numéro de note MIDI (0-127).

---

## 11. Novation Launchpad MK2

Sampleur V2 a un support étendu du **Novation Launchpad MK2** avec contrôle des LEDs en couleur.

### Connexion automatique

Si un Launchpad MK2 est branché au démarrage, il est détecté automatiquement et la carte MIDI par défaut est appliquée (64 pads = 64 touches de la grille 8×8).

### Initialiser le Launchpad

Si les LEDs ne s'allument pas :
1. Sélectionne le bon **Input MIDI** et **Output MIDI** (les deux doivent être le Launchpad)
2. Clique sur **Initialiser Launchpad** (dans le PresetPanel)

### Correspondance des couleurs

La couleur choisie pour chaque pad dans Sampleur correspond à la couleur de la LED sur le Launchpad :

| Couleur Sampleur | LED Launchpad |
|-----------------|--------------|
| Rouge | Rouge |
| Orange | Orange |
| Jaune | Jaune |
| Vert | Vert |
| Cyan | Cyan clair |
| Bleu | Bleu |
| Violet | Violet |
| Rose | Rose |

### Retour visuel pendant la lecture

- **Couleur du pad** : le pad est chargé mais pas en cours de lecture
- **Blanc** : le pad est en cours de lecture

### Rafraîchir les LEDs

Après un chargement de preset, les LEDs se mettent à jour automatiquement. Si elles ne correspondent plus, clique sur **Rafraîchir LEDs**.

---

## 12. Enregistrement live

L'enregistrement capture le **mix final** (tous les pads + effets FX) en qualité lossless.

### Démarrer l'enregistrement

1. Clique sur le bouton **● REC** en haut à droite
2. Le bouton passe en mode **■ STOP REC** avec animation clignotante
3. Un timer affiche le temps écoulé (MM:SS)

### Arrêter l'enregistrement

1. Clique sur **■ STOP REC**
2. Une notification indique le chemin du fichier sauvegardé

### Emplacement des fichiers

Les enregistrements sont sauvegardés automatiquement dans :
```
~/Sampleur-Recordings/Sampleur_AAAA-MM-JJ_HH-MM-SS.wav
```

### Format du fichier

- Format : **WAV 32-bit float**
- Canaux : **Stéréo**
- Fréquence : **48 000 Hz** (qualité studio)
- Taille indicative : ~22 Mo/minute

> **Astuce :** Lance l'enregistrement juste avant de commencer à jouer pour capturer ta session complète.

---

## 13. Réorganiser les pads par glisser-déposer

En **mode CONFIG**, tu peux déplacer les pads pour réorganiser ton kit visuellement.

### Comment faire

1. Active le mode **CONFIG** (le curseur de la grille passe en main)
2. Clique et maintiens sur un pad (il devient semi-transparent)
3. Glisse vers un autre pad (celui-ci se surligne)
4. Relâche : les deux pads échangent leur contenu

### Ce qui est échangé

✅ Échangé : son chargé, nom, couleur, mode, volume, détune, BPM original
❌ Non échangé : la note MIDI assignée (elle reste attachée à la position dans la grille)

> **Pourquoi ?** La note MIDI correspond à une touche physique du contrôleur. Déplacer la note avec le pad créerait une incohérence : appuyer sur une touche physique déclencherait le mauvais pad.

---

## 14. Astuces et bonnes pratiques

### Organiser son kit efficacement

- Utilise les **couleurs** pour regrouper les sons par famille (percussions en rouge, basses en bleu, etc.)
- Le mode **4×4** (16 pads) est idéal pour les kits simples ou les performances live
- Le mode **8×8** (64 pads) permet des kits très complets

### BPM et synchronisation

- Si tu définis un **BPM original** sur un pad, le son sera accéléré ou ralenti pour suivre le BPM global
- Le **Quantize** déclenche les sons exactement sur le temps — parfait pour ne pas jouer en dehors du tempo

### Optimiser les volumes

- Le **volume par pad** (0 à 2) permet d'équilibrer les niveaux entre les sons
- Le **volume master** dans les FX contrôle le niveau global de sortie
- Si le son sature, baisse le volume master plutôt que de baisser chaque pad individuellement

### Utiliser le Delay et la Reverb avec parcimonie

- Delay Mix à 0 et Reverb Mix à 0 = effets complètement coupés (transparents)
- Monte progressivement le mix pour doser l'effet
- Un Delay Feedback > 0.8 peut créer des répétitions très longues

### Sauvegarder souvent

- Utilise le mode **Léger** pour des sauvegardes rapides pendant le travail
- Utilise le mode **Portable** avant de partager ou d'archiver un kit

---

## 15. Dépannage

### Aucun son ne sort

1. Vérifie que ton système audio fonctionne normalement (joue un son dans une autre application)
2. Vérifie que le **Volume Master** dans les FX n'est pas à 0
3. Vérifie que les pads ont bien un son chargé (indicateur vert)
4. Clique sur **■ Stop All** pour s'assurer qu'aucun son ne bloque le moteur

### Le Launchpad n'est pas détecté

1. Branche le Launchpad **avant** de lancer Sampleur
2. Vérifie les sélecteurs MIDI (Input et Output doivent pointer sur le Launchpad)
3. Clique sur **↺** pour rescanner les appareils
4. Clique sur **Initialiser Launchpad**

### Les LEDs du Launchpad ne s'allument pas

1. Vérifie que le **Output MIDI** est bien réglé sur le Launchpad
2. Clique sur **Rafraîchir LEDs** dans le panneau PresetPanel
3. Essaie **Initialiser Launchpad** pour réappliquer le mode programmeur

### Un preset ne retrouve pas ses sons

- Si le preset est en mode **Léger**, les sons doivent se trouver exactement aux mêmes chemins qu'à la sauvegarde
- Si tu as déplacé tes sons, recharge-les manuellement pad par pad
- Pour éviter ce problème à l'avenir, utilise le mode **Portable** lors de la sauvegarde

### La grille est vide après le chargement d'un preset

- Vérifie que les fichiers audio existent encore à leur emplacement d'origine
- Tente de charger le preset depuis un autre emplacement si les fichiers sont dans un dossier `_samples/` à côté du `.sampleur2`

### L'application ne démarre pas (Linux)

- AppImage : vérifie que le fichier est marqué comme exécutable (`chmod +x`)
- .deb : vérifie que les dépendances WebKit2GTK sont installées :
  `sudo apt install libwebkit2gtk-4.1-0`

### Le son est saccadé ou crépite

- Ferme les autres applications audio gourmandes
- Le crépitement peut apparaître si beaucoup de pads jouent simultanément sur un système peu puissant
- Vérifie que ton driver audio est configuré avec une latence correcte (ALSA/PipeWire)

---

*Sampleur V2 — Manuel d'utilisation v2.0.0 — 2026-03-29*

*Pour toute question ou signalement de bug : https://github.com/jbseriziat/sampleur/issues*
