# Connection Monitor Windows Release Design

## Obiettivo

Distribuire Connection Monitor 0.3.1 per Windows 10/11 x64 con installer NSIS `.exe` e WiX `.msi`, mantenendo la release macOS ARM64 e gli aggiornamenti automatici in una sola release GitHub.

## Esperienza Windows

- Monitor, storico locale di 30 giorni, Guardian, Replay, Streak, tooltip, guida e sezione Info restano disponibili.
- La notification area usa una sola icona ufficiale: Windows non mostra i titoli testuali multipli usati dalla menu bar macOS.
- Il click sull’icona apre il pannello principale; il widget flottante mostra download, upload, qualità e dati di sessione.
- L’icona espone un tooltip sintetico con stato e valori correnti.
- Gli installer non sono firmati in questa prima versione; README e note di release spiegano il possibile avviso SmartScreen.

## Architettura multipiattaforma

Il core Rust condiviso continua a gestire banda, ping, qualità, storico e incidenti. Il codice specifico di piattaforma viene isolato dietro `cfg(target_os)`:

- macOS conserva quattro indicatori separati e il rilevamento Wi-Fi esistente;
- Windows crea una sola tray icon e legge il Wi-Fi tramite le API WLAN native;
- le piattaforme senza dati Wi-Fi restituiscono metriche mancanti senza inventare valori o penalizzare la qualità.

La configurazione Tauri abilita i bundle macOS `app`/`dmg` e Windows `nsis`/`msi` attraverso file di configurazione dedicati, evitando opzioni macOS nella build Windows.

## Release GitHub

Un workflow a matrice esegue job indipendenti su `macos-latest` e `windows-latest`. Entrambi usano Node 24, Rust stable, test JavaScript, test Rust e build Tauri. La release `v0.3.1` raccoglie:

- DMG ARM64 macOS;
- updater macOS e firma;
- installer Windows x64 `.exe`;
- installer Windows x64 `.msi`;
- updater Windows e firma;
- `latest.json` multipiattaforma.

La release nasce in bozza e viene pubblicata solo dopo il controllo degli artefatti. Le chiavi updater esistenti firmano i pacchetti Tauri; non viene configurata firma Authenticode.

## Compatibilità e dati

- Minimo supportato: Windows 10 x64 e Windows 11 x64.
- Lo storico resta esclusivamente locale nella directory dati dell’app.
- Il formato storico rimane schema 1, quindi non richiede migrazioni.
- L’updater seleziona automaticamente l’artefatto corretto per sistema e architettura.

## Errori e fallback

- Se WLAN non è disponibile o l’utente usa Ethernet, SSID e segnale restano non disponibili mentre banda e qualità continuano a funzionare.
- Se ICMP è bloccato, resta attivo il fallback TCP esistente.
- Un errore di un job GitHub impedisce la pubblicazione della release ma non cancella gli artefatti già validi.
- Il pannello non deve restare in stato `connecting` dopo il completamento o fallimento della prima sonda.

## Verifiche

- Test unitari dei parser/adattatori di piattaforma e della scelta del modello tray.
- Test JavaScript e Rust completi su macOS e Windows.
- `cargo check` sui target ospitati nativamente dai runner.
- Build Tauri reale di DMG, NSIS e MSI.
- Controllo nomi, dimensioni e firme updater degli artefatti GitHub.
- Smoke test Windows documentato: installazione, avvio, tray, widget, pannello, storico e disinstallazione.

## Fuori ambito

- Firma Authenticode e rimozione dell’avviso SmartScreen.
- Windows ARM64 e sistemi precedenti a Windows 10.
- Modifiche grafiche non necessarie al comportamento nativo Windows.
