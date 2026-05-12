# CardCounter - Festifoll 2026 🎡

⚠️ **PROJET ABANDONNÉ DANS SON ÉTAT ACTUEL** ⚠️  
Le projet est officiellement délaissé par son auteur d'origine. La propriété du dépôt et la maintenance peuvent être transférées à toute personne intéressée. Contactez-moi par issue ou transfert direct.

Outil de comptage rapide par scan de codes pour l'édition **Festi'Foll 2026**.

## 🚀 Fonctionnalités

### ✅ Fonctionnelles
- **Saisie Clavier Ultra-Rapide** : Optimisée pour les lecteurs de codes-barres émulant un clavier.
- **Validation Auto** : Traite les codes par lots de 6 caractères (ex: `032860`).
- **Base de Données** : Stockage local SQLite (`codes.db`).
- **Interface GPU** : UI performante construite avec GPUI 0.2.2.
- **Mode Reset** : Bouton de réinitialisation sécurisé avec confirmation.

### 🚧 WIP / Expérimental (Abstrait)
- **Scanner Caméra (Beta)** : Intégration de `nokhwa` et `rxing` pour le scan direct via webcam.
  - Stream vidéo ~15 FPS.
  - HUD avec overlay assombri et zone cible pour faciliter le focus.
  - Support Code 128, EAN-13, et Interleaved 2 of 5 (ITF).
  - *Note : Le décodage ITF est actuellement capricieux sur certains matériels.*

## 🛠 Installation / Utilisation

### Windows (Release)
1. Téléchargez le dernier `.exe` depuis l'onglet **Releases**.
2. Lancez `card-counter.exe`.

### Développement (Compilation source)
Pré-requis : [Rust](https://rustup.rs/) installé.
```bash
cargo run --release
```

## ⌨️ Raccourcis & UX
- Le programme prend le focus automatiquement au démarrage.
- **Entrée de code** : Tapez ou scannez directement (chiffres uniquement).
- **Reset DB** : Cliquez sur le bouton "RESET" en bas à droite, puis confirmez sur l'overlay rouge.
