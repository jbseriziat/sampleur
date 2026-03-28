#!/bin/bash

# 1. Se placer dans le dossier où se trouve ce script (et le fichier html)
cd "$(dirname "$0")"

# 2. Vérifier si Python 3 est installé (il l'est toujours sur Ubuntu)
if command -v python3 &>/dev/null; then
    echo "Démarrage du mini-serveur musical..."
    
    # 3. Lancer un serveur web léger en arrière-plan sur le port 8000
    # Cela permet de tromper le navigateur pour qu'il croie être sur un vrai site web
    # et autorise ainsi l'accès complet au MIDI.
    python3 -m http.server 8000 &
    PID=$! # On garde l'ID du processus pour pouvoir le tuer plus tard
    
    # 4. Attendre 1 seconde que le serveur démarre
    sleep 1
    
    # 5. Ouvrir le navigateur par défaut (Chrome/Chromium recommandé)
    # Assurez-vous que le fichier HTML s'appelle bien Sampleur_V11.html
    xdg-open "http://localhost:8000/Sampleur_V11.html"
    
    echo "Appuyez sur une touche pour arrêter le serveur quand vous avez fini."
    read -n 1
    
    # 6. Nettoyage quand on ferme la fenêtre du terminal
    kill $PID
else
    # Fallback si python n'est pas là (rare) : ouverture directe
    # Risque : le MIDI peut être bloqué par le navigateur
    xdg-open "Sampleur_V11.html"
fi