# Connection Monitor 0.2.0 design

## Obiettivo

Rendere tray e widget coerenti, leggibili e affidabili. La tray deve mostrare quattro indicatori distinti: download, upload, qualita e dati trasferiti nella sessione. L'icona composita attuale viene eliminata.

## Tray

La tray usa quattro status item Tauri adiacenti, tutti cliccabili per aprire o chiudere lo stesso widget:

- download: freccia verde `#34D399`, velocita adattiva Kbps/Mbps/Gbps;
- upload: freccia blu `#60A5FA`, velocita adattiva Kbps/Mbps/Gbps;
- qualita: anello dinamico verde, giallo, arancio o rosso e punteggio `/100`;
- dati: cilindro viola `#A78BFA` e totale MB/GB/TB trasferito dall'avvio dell'app.

Non resta alcuna icona generica prima degli indicatori. Ogni icona e disegnata a 18 x 18 pixel con trasparenza e colori pieni, senza emoji.

## Snapshot unico

Il backend misura la rete una sola volta al secondo e salva un `ConnectionStats` completo. `get_bandwidth`, eventi frontend, tray, grafico e footer leggono lo stesso snapshot. Il comando frontend non deve chiamare nuovamente `measure()`.

I dati trasferiti sono delta di sessione rispetto ai contatori iniziali delle interfacce, non traffico storico precedente all'avvio.

## Connessione e qualita

Lo stato e esplicito: `connecting` prima del primo probe, `online` dopo un probe riuscito e `offline` dopo un probe fallito. SSID Wi-Fi, traffico storico e contatori cumulativi non sono prove di accesso Internet.

La latenza corrente usa l'ultimo probe, anche quando fallisce. Il punteggio visualizzato applica smoothing esponenziale al punteggio grezzo per evitare oscillazioni; lo stato offline forza punteggio zero. Le notifiche di qualita richiedono una variazione significativa.

## Widget

Le velocita sono indicate correttamente in bit per secondo con unita adattive. Il prefisso errato `MB/s` viene rimosso. Il footer mostra i dati trasferiti nella sessione. La UI usa direttamente il payload `stats-update` e mantiene il recupero iniziale tramite `get_bandwidth`.

## Versione e distribuzione

La versione diventa `0.2.0` in Tauri, Cargo e npm. Il bundle locale usa firma ad-hoc valida. Il workflow GitHub crea artefatti updater firmati e `latest.json` usando i secret Tauri; supporta firma Developer ID e notarizzazione quando i relativi secret Apple sono configurati.

Sul Mac non e installata alcuna identita Developer ID: la release locale non puo essere notarizzata da Apple. Questa limitazione non viene nascosta o simulata.

## Verifica

- Test unitari Rust per contatori di sessione, ultimo probe, smoothing, formattazione tray e pixel delle quattro icone.
- Test JavaScript per unita adattive e dati totali.
- Test completi, build Apple Silicon, firma, confronto dei bundle, verifica DMG e processo realmente avviato da `/Applications`.
- Controllo visivo finale di tray e widget con screenshot.
