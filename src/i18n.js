const translations = {
    en: {
        appTitle: 'Connection Monitor',
        statusConnecting: 'Connecting...',
        statusDisconnected: 'Disconnected',
        speedDown: 'down',
        speedUp: 'up',
        chartTitle: 'Bandwidth (last 60s)',
        chartLegendDown: 'Down',
        chartLegendUp: 'Up',
        metricPing: 'Ping',
        metricJitter: 'Jitter',
        metricLoss: 'Loss',
        metricWifi: 'WiFi',
        wifiEthernet: 'Ethernet',
        btnWidget: 'Widget',
        btnSpeedTest: 'Speed Test',
        speedTestReady: 'Click "Speed Test" to measure',
        speedTestConnecting: 'Connecting to server...',
        speedTestTesting: 'Testing...',
        speedTestLatency: 'Latency',
        speedTestError: 'Test failed',
        speedTestMbps: 'Mbps',
        uptime: 'Uptime',
        totalData: 'Total',
        qualityExcellent: 'Excellent',
        qualityGood: 'Good',
        qualityFair: 'Fair',
        qualityPoor: 'Poor',
        qualityCritical: 'Critical',
        language: 'Language',
        langEn: 'English',
        langIt: 'Italiano',
        langEs: 'Espanol',
        langFr: 'Francais',
        timeH: 'h',
        timeM: 'm',
        timeS: 's',
        trayHide: 'Hide',
        settings: 'Settings',
        closeBtn: 'Close',
        tabMonitor: 'Monitor',
        tabCredits: 'Credits'
    },
    it: {
        appTitle: 'Monitor Connessione',
        statusConnecting: 'Connessione...',
        statusDisconnected: 'Disconnesso',
        speedDown: 'in arrivo',
        speedUp: 'in uscita',
        chartTitle: 'Banda (ultimi 60s)',
        chartLegendDown: 'Scaric.',
        chartLegendUp: 'Invio',
        metricPing: 'Ping',
        metricJitter: 'Jitter',
        metricLoss: 'Persi',
        metricWifi: 'WiFi',
        wifiEthernet: 'Ethernet',
        btnWidget: 'Widget',
        btnSpeedTest: 'Test Velocita',
        speedTestReady: 'Clicca "Test Velocita" per misurare',
        speedTestConnecting: 'Connessione al server...',
        speedTestTesting: 'Test in corso...',
        speedTestLatency: 'Latenza',
        speedTestError: 'Test fallito',
        speedTestMbps: 'Mbps',
        uptime: 'Attivo da',
        totalData: 'Totale',
        qualityExcellent: 'Eccellente',
        qualityGood: 'Buono',
        qualityFair: 'Discreto',
        qualityPoor: 'Scadente',
        qualityCritical: 'Critico',
        language: 'Lingua',
        langEn: 'English',
        langIt: 'Italiano',
        langEs: 'Espanol',
        langFr: 'Francais',
        timeH: 'h',
        timeM: 'm',
        timeS: 's',
        trayHide: 'Nascondi',
        settings: 'Impostazioni',
        closeBtn: 'Chiudi',
        tabMonitor: 'Monitor',
        tabCredits: 'Crediti'
    },
    es: {
        appTitle: 'Monitor de Conexion',
        statusConnecting: 'Conectando...',
        statusDisconnected: 'Desconectado',
        speedDown: 'bajada',
        speedUp: 'subida',
        chartTitle: 'Ancho de banda (ultimos 60s)',
        chartLegendDown: 'Bajada',
        chartLegendUp: 'Subida',
        metricPing: 'Ping',
        metricJitter: 'Jitter',
        metricLoss: 'Perd.',
        metricWifi: 'WiFi',
        wifiEthernet: 'Ethernet',
        btnWidget: 'Widget',
        btnSpeedTest: 'Test Velocidad',
        speedTestReady: 'Haz clic en "Test Velocidad" para medir',
        speedTestConnecting: 'Conectando al servidor...',
        speedTestTesting: 'Probando...',
        speedTestLatency: 'Latencia',
        speedTestError: 'Prueba fallida',
        speedTestMbps: 'Mbps',
        uptime: 'Activo',
        totalData: 'Total',
        qualityExcellent: 'Excelente',
        qualityGood: 'Bueno',
        qualityFair: 'Regular',
        qualityPoor: 'Malo',
        qualityCritical: 'Critico',
        language: 'Idioma',
        langEn: 'English',
        langIt: 'Italiano',
        langEs: 'Espanol',
        langFr: 'Francais',
        timeH: 'h',
        timeM: 'm',
        timeS: 's',
        trayHide: 'Ocultar',
        settings: 'Ajustes',
        closeBtn: 'Cerrar',
        tabMonitor: 'Monitor',
        tabCredits: 'Creditos'
    },
    fr: {
        appTitle: 'Moniteur Connexion',
        statusConnecting: 'Connexion...',
        statusDisconnected: 'Deconnecte',
        speedDown: 'descendant',
        speedUp: 'montant',
        chartTitle: 'Bande passante (60s dern.)',
        chartLegendDown: 'Desc.',
        chartLegendUp: 'Mont.',
        metricPing: 'Ping',
        metricJitter: 'Jitter',
        metricLoss: 'Pertes',
        metricWifi: 'WiFi',
        wifiEthernet: 'Ethernet',
        btnWidget: 'Widget',
        btnSpeedTest: 'Test de debit',
        speedTestReady: 'Cliquez sur "Test de debit" pour mesurer',
        speedTestConnecting: 'Connexion au serveur...',
        speedTestTesting: 'Test en cours...',
        speedTestLatency: 'Latence',
        speedTestError: 'Echec du test',
        speedTestMbps: 'Mbps',
        uptime: 'Actif depuis',
        totalData: 'Total',
        qualityExcellent: 'Excellent',
        qualityGood: 'Bon',
        qualityFair: 'Correct',
        qualityPoor: 'Faible',
        qualityCritical: 'Critique',
        language: 'Langue',
        langEn: 'English',
        langIt: 'Italiano',
        langEs: 'Espanol',
        langFr: 'Francais',
        timeH: 'h',
        timeM: 'min',
        timeS: 's',
        trayHide: 'Masquer',
        settings: 'Parametres',
        closeBtn: 'Fermer',
        tabMonitor: 'Moniteur',
        tabCredits: 'Credits'
    }
};

class I18n {
    constructor() {
        this.currentLang = 'en';
        this.listeners = [];
    }

    setLanguage(lang) {
        if (this.currentLang === lang) return;
        if (!translations[lang]) return;
        this.currentLang = lang;
        document.documentElement.lang = lang;
        this.listeners.forEach(fn => fn(lang));
    }

    t(key) {
        return translations[this.currentLang]?.[key] ?? translations.en[key] ?? key;
    }

    getLanguage() {
        return this.currentLang;
    }

    onChange(fn) {
        this.listeners.push(fn);
    }
}

const i18n = new I18n();

window.i18n = i18n;
export { i18n };
