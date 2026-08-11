# Correzione tray, widget e stato connessione

## Obiettivo

Correggere tre problemi su macOS:

- il widget scompare al rilascio del click;
- gli indicatori dinamici non sono leggibili nella barra dei menu;
- l'interfaccia resta su "Connecting...".

La build aggiornata deve sostituire sia `/Applications/Connection Monitor.app` sia la copia in `Connection Monitor/ULTIMA VERSIONE`.

## Comportamento richiesto

- Un click sinistro sull'icona apre il widget.
- Il rilascio del mouse non lo chiude.
- Il widget resta aperto finche l'utente preme il pulsante di chiusura oppure riclicca l'icona della barra dei menu.
- La barra dei menu mostra sempre un'icona visibile e aggiornata.
- Lo stato iniziale `Connecting...` viene sostituito appena arriva il primo campione; in caso di errore viene mostrato uno stato esplicito, non un'attesa infinita.

## Soluzione

La gestione apertura/chiusura resta nel backend Tauri. Il frontend non chiude piu la finestra in risposta a click generici nel documento. Il tray reagisce una sola volta al click sinistro completo e usa un'icona adatta alla barra dei menu macOS, con fallback sempre valido.

Il monitor di connessione produce uno stato deterministico dopo il primo tentativo. Il ping ICMP mantiene un fallback TCP per gli ambienti dove ICMP non e disponibile. Backend e frontend condividono gli stessi criteri per connesso, disconnesso e qualita.

## Verifica

- Test automatici per stato iniziale, connessione e fallback dell'icona.
- Build release pulita per Apple Silicon.
- Confronto hash tra binario compilato, copia `ULTIMA VERSIONE` e copia installata.
- Terminazione delle istanze precedenti, avvio da `/Applications` e verifica del percorso del processo realmente in esecuzione.
- Controllo visivo dell'icona e del widget: apertura, rilascio, permanenza, riclick e pulsante di chiusura.

## Limiti

Nessuna nuova funzione e nessun rifacimento AppKit. Le modifiche restano circoscritte a tray, ciclo del monitor e stato visivo.
