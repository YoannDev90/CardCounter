# CardCounter - Festifoll 2026 🎡

Outil de comptage rapide par scan de codes pour l'édition **Festifoll 2026**.

## 🚀 Fonctionnalités
- **Scan Ultra-Rapide** : Optimisé pour les lecteurs de codes-barres (entrée clavier série).
- **Validation Auto** : Traite les codes par lots de 6 caractères.
- **Base de Données** : Stockage local SQLite (`codes.db`).
- **Mode Reset** : Bouton de réinitialisation sécurisé avec confirmation.

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
