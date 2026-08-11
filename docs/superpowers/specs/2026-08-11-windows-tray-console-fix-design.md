# Windows Tray and Console Fix Design

## Obiettivo

Correggere Connection Monitor su Windows affinche l'app funzioni come applicazione grafica autonoma, senza console, e mostri nella notification area quattro indicatori distinti per download, upload, qualita e dati trasferiti.

## Comportamento Windows

- La build release usa il sottosistema Windows GUI e non apre una console nera.
- La chiusura di eventuali terminali usati per avviare l'app non termina l'app installata.
- La tray contiene esattamente quattro icone colorate: download, upload, qualita e dati.
- Ogni icona aggiorna colore e simbolo in tempo reale e mostra il valore numerico nel proprio tooltip.
- Il clic sinistro su qualsiasi indicatore apre o chiude il pannello principale.
- Non viene aggiunta una quinta icona con il logo, per evitare rumore nella tray.

Windows decide autonomamente se mostrare le nuove icone accanto all'orologio oppure nel menu delle icone nascoste. L'app non puo forzare il pin; la guida deve spiegare come renderle sempre visibili dalle impostazioni della barra delle applicazioni.

## Implementazione

`src-tauri/src/main.rs` dichiara `windows_subsystem = "windows"` solo per le build non-debug su Windows, mantenendo disponibile la console durante lo sviluppo.

La selezione della tray non usa piu la modalita Windows a icona singola. macOS e Windows condividono i quattro indicatori, mentre le differenze non supportate vengono isolate:

- macOS continua a mostrare anche il titolo testuale accanto alle icone;
- Windows usa quattro icone e tooltip, senza dipendere dai titoli testuali non visualizzati dalla notification area;
- le preferenze colore esistenti restano la fonte dei colori su entrambe le piattaforme.

## Verifiche

- Test statico che impedisce la regressione della dichiarazione `windows_subsystem`.
- Test Rust che richiede quattro indicatori su Windows.
- Test Rust sui tooltip e sulle icone colorate.
- Suite JavaScript e Rust completa su macOS.
- GitHub Actions Windows con test, compilazione e produzione reale degli installer NSIS e MSI.
- Controllo degli artefatti e pubblicazione della release `v0.3.2` solo dopo CI verde.

## Fuori ambito

- Forzare il pin delle icone accanto all'orologio, non consentito dalle API Windows.
- Firma Authenticode degli installer.
- Modifiche al calcolo delle metriche di rete o alla UI del pannello.
