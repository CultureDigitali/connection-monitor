use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    En,
    It,
    Es,
    Fr,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "it" => Language::It,
            "es" => Language::Es,
            "fr" => Language::Fr,
            _ => Language::En,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::It => "it",
            Language::Es => "es",
            Language::Fr => "fr",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::It => "Italiano",
            Language::Es => "Espanol",
            Language::Fr => "Francais",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::En
    }
}

pub struct I18n {
    lang: Arc<Mutex<Language>>,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self {
            lang: Arc::new(Mutex::new(lang)),
        }
    }

    pub fn get(&self) -> Language {
        *self.lang.lock()
    }

    pub fn set(&self, lang: Language) {
        *self.lang.lock() = lang;
    }

    pub fn t(&self, key: &str) -> String {
        let lang = self.get();
        translate(lang, key)
    }
}

pub fn translate(lang: Language, key: &str) -> String {
    let s: &str = match key {
        "app_title" => match lang {
            Language::En => "Connection Monitor",
            Language::It => "Monitor Connessione",
            Language::Es => "Monitor de Conexion",
            Language::Fr => "Moniteur Connexion",
        },
        "quality_excellent" => match lang {
            Language::En => "Excellent",
            Language::It => "Eccellente",
            Language::Es => "Excelente",
            Language::Fr => "Excellent",
        },
        "quality_good" => match lang {
            Language::En => "Good",
            Language::It => "Buono",
            Language::Es => "Bueno",
            Language::Fr => "Bon",
        },
        "quality_fair" => match lang {
            Language::En => "Fair",
            Language::It => "Discreto",
            Language::Es => "Regular",
            Language::Fr => "Correct",
        },
        "quality_poor" => match lang {
            Language::En => "Poor",
            Language::It => "Scadente",
            Language::Es => "Malo",
            Language::Fr => "Faible",
        },
        "quality_critical" => match lang {
            Language::En => "Critical",
            Language::It => "Critico",
            Language::Es => "Critico",
            Language::Fr => "Critique",
        },
        "quality_disconnected" => match lang {
            Language::En => "Disconnected",
            Language::It => "Disconnesso",
            Language::Es => "Desconectado",
            Language::Fr => "Deconnecte",
        },
        "quality_connecting" => match lang {
            Language::En => "Connecting...",
            Language::It => "Connessione...",
            Language::Es => "Conectando...",
            Language::Fr => "Connexion...",
        },
        "notif_conn_lost_title" => match lang {
            Language::En => "Connection Lost",
            Language::It => "Connessione Persa",
            Language::Es => "Conexion Perdida",
            Language::Fr => "Connexion Perdue",
        },
        "notif_conn_lost_body" => match lang {
            Language::En => "Internet connection has been lost",
            Language::It => "La connessione internet e stata persa",
            Language::Es => "Se ha perdido la conexion a internet",
            Language::Fr => "La connexion internet a ete perdue",
        },
        "notif_conn_restored_title" => match lang {
            Language::En => "Connection Restored",
            Language::It => "Connessione Ripristinata",
            Language::Es => "Conexion Restaurada",
            Language::Fr => "Connexion Restauree",
        },
        "notif_conn_restored_body" => match lang {
            Language::En => "Internet connection is back",
            Language::It => "La connessione internet e tornata",
            Language::Es => "La conexion a internet ha vuelto",
            Language::Fr => "La connexion internet est revenue",
        },
        "notif_degraded_title" => match lang {
            Language::En => "Connection Degraded",
            Language::It => "Connessione Peggiorata",
            Language::Es => "Conexion Degradada",
            Language::Fr => "Connexion Degradee",
        },
        "notif_improved_title" => match lang {
            Language::En => "Connection Improved",
            Language::It => "Connessione Migliorata",
            Language::Es => "Conexion Mejorada",
            Language::Fr => "Connexion Amelioree",
        },
        "notif_quality_body" => match lang {
            Language::En => "Quality is now",
            Language::It => "La qualita e ora",
            Language::Es => "La calidad ahora es",
            Language::Fr => "La qualite est maintenant",
        },
        _ => "",
    };
    if s.is_empty() {
        key.to_string()
    } else {
        s.to_string()
    }
}
