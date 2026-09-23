# Checklist de tests manuels (hors fichiers/CI)

État actuel de validation:

- [x] Validation automatisée Rust : `cargo test --workspace --quiet` est verte
- [x] Vérification des règles d’export navigateur : FLAC par défaut via le bridge Rust/WASM, WAV et MP3 320 kbps sélectionnables, pas de renommage silencieux
- [x] Vérification de la génération de session et du packaging ZIP sur les fichiers actifs
- [x] Validation finale sur mixeur Ui24R réel : FLAC et WAV fonctionnent (multitrack complet) ; MP3 échoue avec une erreur de session même après correction manuelle de l’extension
- [ ] Validation contre le jeu de fixtures officiel pour une compatibilité byte-à-byte complète
- [x] Validation du flux de conversion FLAC côté navigateur via Rust/WASM (bundle généré et chargé, conversion source et split stéréo vérifiés, fallback vérifié quand le module n’est pas présent)
- [x] Validation de l’encodage MP3 320 kbps côté CLI et du bridge Wasm
- [x] Validation CI de la présence du bundle Web, des exports Wasm FLAC/MP3 et des actions stéréo
- [x] Validation CI Rust : formatage, tests, Clippy, documentation et build Wasm frais
- [x] Vérification manuelle de l’interface Web dans Firefox

1. Sur ton PC (CLI Rust)

- cargo test --workspace + cargo clippy ... -D warnings déjà automatisés, mais en plus, teste à la main :
    - ui24-session-builder analyze <fichier> sur tes vrais fichiers tmp/wav/*.wav et tmp/mp3/*.mp3 — vérifie que le sample rate, la durée et le format affichés correspondent à la réalité (ouvre le fichier dans un lecteur/éditeur audio pour comparer).
    - ui24-session-builder convert <in> <out.flac> — écoute le FLAC généré pour confirmer qu'il n'y a pas de distorsion, de décalage ou de silence inattendu.
    - Tester une source WAV 24-bit longue (par exemple `Complainiacs_Etc_Full.wav/01_Kick.wav`) puis relancer `analyze` sur le FLAC généré : il doit être lisible et conserver 24 bits, le sample rate et la durée.
    - ui24-session-builder convert <dossier-audio> <dossier-sortie> --format flac — vérifie que tous les WAV/FLAC/AIFF/MP3 du premier niveau sont convertis, que les fichiers non audio et les sous-dossiers sont ignorés, et que les noms conservent leur stem avec l’extension cible.
    - Vérifie qu’une destination existante qui est un fichier est refusée, tandis qu’une destination absente est créée comme dossier.
    - ui24-session-builder create <dossier> <sortie> --zip avec un dossier contenant plusieurs pistes à sample rates différents — vérifie que l'outil refuse proprement (pas de crash).
    - ui24-session-builder create <dossier> sans sortie — vérifie que seul `<dossier>/.uirecsession` est écrit, que les fichiers audio ne sont ni copiés ni convertis, que `--zip` sans sortie est refusé, et que le champ `ext` correspond à l’extension réelle des fichiers du dossier (pas à `--format`, ignoré dans ce mode avec une note affichée). Vérifie qu’un dossier mélangeant plusieurs extensions audio est refusé proprement.
    - Ouvre le .uirecsession généré dans un éditeur de texte pour vérifier visuellement les champs (files, names, mapping, sampleRate, lengthSeconds).

2. Dans un navigateur desktop (Web UI)

- Firefox : validation manuelle de l’interface, de l’analyse locale et des actions stéréo effectuée.
- Vérifier le menu repliable "How to prepare the USB key for the Ui24R" et que les instructions correspondent bien à la structure FAT32/Multitrack/session-folder demandée par le mixer.
- Glisser-déposer plusieurs fichiers audio (WAV, MP3, mélange) et vérifier que la durée/sample rate/extension affichés sont corrects.
- Vérifier que l’analyse d’un WAV/AIFF/MP3 affiche rapidement ses métadonnées sans conversion et que le sélecteur est positionné sur `FLAC (WASM)`.
- Vérifier que `Extension` et les noms du tableau affichent `.flac` comme projection avant conversion, puis afficher `.wav` après sélection de WAV sans relancer l’analyse.
- Sélectionner `MP3 (320 kbps)`, exporter un ZIP et vérifier que les fichiers audio portent l’extension `.mp3` et sont lisibles par la commande `analyze`.
- Cliquer sur `Download session .zip` puis vérifier que le badge de statut affiche `Converting 1/N`, `Converting 2/N`, etc., et que le ZIP contient les fichiers `.flac`.
- Tester un vrai fichier stéréo avec canaux différents (ex. musique stéréo normale) → doit afficher deux pistes L/R sans générer de WAV pendant l’analyse, puis produire `<nom> L.flac` / `<nom> R.flac` à l’export.
- Vérifier que les deux pistes issues du split restent immédiatement éditables, puis que le ZIP les contient sous les noms `<nom> L.flac` / `<nom> R.flac` après export.
- Tester un fichier stéréo dupliqué (mêmes canaux L/R) → doit afficher une seule piste mono par défaut, sans générer de WAV avant l’export.
- Sur une piste stéréo, cliquer `Downmix mono` dans la colonne Action : l’affichage doit remplacer L/R par une seule piste mono. Le bouton devient `Split stereo`; cliquer dessus doit restaurer les deux pistes. Vérifier que l’audio n’est réellement converti qu’au ZIP.
- Modifier les noms de pistes et les mappings dans le tableau, puis télécharger le .uirecsession → vérifier le contenu.
- Vérifier que le fichier téléchargé s'appelle exactement `.uirecsession` (pas `session.uirecsession`) — c'est le nom exact attendu par le Ui24R.
- Cliquer sur "Download session .zip" → décompresser l'archive et vérifier qu'elle contient bien les fichiers audio + un fichier `.uirecsession` (nom exact, pas `session.uirecsession`) lisibles.
- Retirer une piste (bouton "Remove", par exemple une moitié "L" d'un split stéréo) puis télécharger le ZIP → vérifier que le fichier audio retiré n'est PAS présent dans l'archive (seuls les fichiers restants doivent y être).
- Recharger un .uirecsession existant (glisser un fichier JSON) et vérifier que l'inspection fonctionne.
- Tester avec la fenêtre réduite (mobile-width) pour vérifier la mise en page réactive.

3. Sur Android (navigateur mobile, Firefox/Chrome)

- Ouvrir la page GitHub Pages ou le déploiement local sur le téléphone.
- Cliquer sur "Choose or drop files" → vérifier qu'aucune demande de permission microphone n'apparaît (c'était le bug corrigé précédemment).
- Sélectionner plusieurs fichiers audio depuis le stockage du téléphone et vérifier que l'analyse (durée/sample rate) fonctionne aussi bien que sur desktop.
- Tester le téléchargement du .uirecsession et du .zip — vérifier où Android les enregistre (dossier Téléchargements) et qu'ils s'ouvrent correctement.
- Vérifier que le fichier téléchargé se nomme bien `.uirecsession` (certains gestionnaires de fichiers Android masquent les fichiers commençant par un point : vérifier qu'il apparaît bien dans le dossier Téléchargements, au besoin en activant "afficher les fichiers cachés").
- Tester en rotation portrait/paysage.

4. Sur le vrai mixeur Ui24R (le test le plus important, non automatisable)

- Générer une session complète (dossier ou ZIP) via la CLI ou le Web UI avec de vraies pistes.
- Copier la session sur une carte SD (ou clé USB selon le transport supporté par le Ui24R).
- Charger la session directement sur le mixeur et vérifier :
    - que le Ui24R accepte le fichier sans erreur ni refus ;
    - que le nombre de pistes correspond ;
    - que le mapping des canaux (i.0 à i.21) correspond bien à ce qui a été configuré côté logiciel ;
    - que la lecture audio est correcte (pas de désynchronisation, pas de bruit, bon ordre stéréo si un split L/R a été fait) ;
    - que les noms de pistes s'affichent correctement sur l'écran du mixeur (attention aux caractères accentués/Unicode).
- Comparer avec une session générée par l'outil officiel Windows (si disponible) pour repérer d'éventuelles différences byte-à-byte ou de comportement.
- Résultat observé (2026-09) : FLAC et WAV fonctionnent sur un vrai Ui24R multitrack. MP3 provoque une erreur de session sur le mixeur, même après correction manuelle du champ `ext` dans `.uirecsession` — à éviter tant que la cause exacte (probablement le padding/délai introduit par l'encodeur MP3) n'est pas résolue.
