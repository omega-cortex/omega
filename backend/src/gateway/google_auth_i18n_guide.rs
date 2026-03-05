//! Setup guide message for the `/google` credential flow.
//!
//! Extracted from `google_auth_i18n.rs` to respect the 500-line-per-file rule.
//! Contains the lengthy project-specific setup guide with API links.

use super::google_auth_oauth::{gcp_api_library_url, gcp_console_url};

/// Step 2: Comprehensive setup guide with project-specific links.
pub(super) fn google_step_setup_guide_message(lang: &str, project_id: &str) -> String {
    let gmail_url = gcp_api_library_url(project_id, "gmail.googleapis.com");
    let calendar_url = gcp_api_library_url(project_id, "calendar-json.googleapis.com");
    let drive_url = gcp_api_library_url(project_id, "drive.googleapis.com");
    let docs_url = gcp_api_library_url(project_id, "docs.googleapis.com");
    let sheets_url = gcp_api_library_url(project_id, "sheets.googleapis.com");
    let consent_url = gcp_console_url(project_id, "apis/credentials/consent");
    let cred_url = gcp_console_url(project_id, "apis/credentials/oauthclient");

    match lang {
        "Spanish" => format!(
            "Proyecto recibido: *{project_id}*\n\n\
             Sigue estos pasos:\n\n\
             *1. Habilitar APIs* (haz clic en cada enlace y activa):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. Pantalla de consentimiento OAuth*\n\
             {consent_url}\n\
             - Haz clic en \"Get Started\"\n\
             - Nombre: omega | Email: tu email\n\
             - Audiencia: External | Crea\n\n\
             *3. Crear credenciales OAuth*\n\
             {cred_url}\n\
             - Tipo: Web application\n\
             - URI de redireccion: https://omgagi.ai/oauth/callback/\n\
             - *Descarga el JSON*\n\n\
             *4. Publicar la app*\n\
             {consent_url}\n\
             - Ve a \"Audience\" y haz clic en \"Publish App\"\n\n\
             *Pega el contenido completo del archivo JSON descargado cuando estes listo.*"
        ),
        "Portuguese" => format!(
            "Projeto recebido: *{project_id}*\n\n\
             Siga estes passos:\n\n\
             *1. Habilitar APIs* (clique em cada link e ative):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. Tela de consentimento OAuth*\n\
             {consent_url}\n\
             - Clique em \"Get Started\"\n\
             - Nome: omega | Email: seu email\n\
             - Audiencia: External | Crie\n\n\
             *3. Criar credenciais OAuth*\n\
             {cred_url}\n\
             - Tipo: Web application\n\
             - URI de redirecionamento: https://omgagi.ai/oauth/callback/\n\
             - *Baixe o JSON*\n\n\
             *4. Publicar o app*\n\
             {consent_url}\n\
             - Va a \"Audience\" e clique em \"Publish App\"\n\n\
             *Cole o conteudo completo do arquivo JSON baixado quando estiver pronto.*"
        ),
        "French" => format!(
            "Projet recu : *{project_id}*\n\n\
             Suivez ces etapes :\n\n\
             *1. Activer les APIs* (cliquez sur chaque lien et activez) :\n\
             - Gmail : {gmail_url}\n\
             - Calendar : {calendar_url}\n\
             - Drive : {drive_url}\n\
             - Docs : {docs_url}\n\
             - Sheets : {sheets_url}\n\n\
             *2. Ecran de consentement OAuth*\n\
             {consent_url}\n\
             - Cliquez sur \"Get Started\"\n\
             - Nom : omega | Email : votre email\n\
             - Audience : External | Creez\n\n\
             *3. Creer des identifiants OAuth*\n\
             {cred_url}\n\
             - Type : Web application\n\
             - URI de redirection : https://omgagi.ai/oauth/callback/\n\
             - *Telechargez le JSON*\n\n\
             *4. Publier l'app*\n\
             {consent_url}\n\
             - Allez a \"Audience\" et cliquez sur \"Publish App\"\n\n\
             *Collez le contenu complet du fichier JSON telecharge quand vous etes pret.*"
        ),
        "German" => format!(
            "Projekt erhalten: *{project_id}*\n\n\
             Folge diesen Schritten:\n\n\
             *1. APIs aktivieren* (klicke auf jeden Link und aktiviere):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. OAuth-Zustimmungsbildschirm*\n\
             {consent_url}\n\
             - Klicke auf \"Get Started\"\n\
             - Name: omega | E-Mail: deine E-Mail\n\
             - Zielgruppe: External | Erstellen\n\n\
             *3. OAuth-Zugangsdaten erstellen*\n\
             {cred_url}\n\
             - Typ: Web application\n\
             - Weiterleitungs-URI: https://omgagi.ai/oauth/callback/\n\
             - *Lade die JSON-Datei herunter*\n\n\
             *4. App veroffentlichen*\n\
             {consent_url}\n\
             - Gehe zu \"Audience\" und klicke auf \"Publish App\"\n\n\
             *Fuge den vollstandigen Inhalt der heruntergeladenen JSON-Datei ein, wenn du bereit bist.*"
        ),
        "Italian" => format!(
            "Progetto ricevuto: *{project_id}*\n\n\
             Segui questi passaggi:\n\n\
             *1. Abilitare le API* (clicca su ogni link e attiva):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. Schermata di consenso OAuth*\n\
             {consent_url}\n\
             - Clicca su \"Get Started\"\n\
             - Nome: omega | Email: la tua email\n\
             - Pubblico: External | Crea\n\n\
             *3. Creare credenziali OAuth*\n\
             {cred_url}\n\
             - Tipo: Web application\n\
             - URI di reindirizzamento: https://omgagi.ai/oauth/callback/\n\
             - *Scarica il JSON*\n\n\
             *4. Pubblicare l'app*\n\
             {consent_url}\n\
             - Vai a \"Audience\" e clicca su \"Publish App\"\n\n\
             *Incolla il contenuto completo del file JSON scaricato quando sei pronto.*"
        ),
        "Dutch" => format!(
            "Project ontvangen: *{project_id}*\n\n\
             Volg deze stappen:\n\n\
             *1. API's inschakelen* (klik op elke link en activeer):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. OAuth-toestemmingsscherm*\n\
             {consent_url}\n\
             - Klik op \"Get Started\"\n\
             - Naam: omega | E-mail: je e-mail\n\
             - Doelgroep: External | Maken\n\n\
             *3. OAuth-inloggegevens maken*\n\
             {cred_url}\n\
             - Type: Web application\n\
             - Omleidings-URI: https://omgagi.ai/oauth/callback/\n\
             - *Download de JSON*\n\n\
             *4. App publiceren*\n\
             {consent_url}\n\
             - Ga naar \"Audience\" en klik op \"Publish App\"\n\n\
             *Plak de volledige inhoud van het gedownloade JSON-bestand wanneer je klaar bent.*"
        ),
        "Russian" => format!(
            "Проект получен: *{project_id}*\n\n\
             Выполните следующие шаги:\n\n\
             *1. Включить API* (нажмите на каждую ссылку и активируйте):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. Экран согласия OAuth*\n\
             {consent_url}\n\
             - Нажмите \"Get Started\"\n\
             - Название: omega | Email: ваш email\n\
             - Аудитория: External | Создать\n\n\
             *3. Создать учетные данные OAuth*\n\
             {cred_url}\n\
             - Тип: Web application\n\
             - URI перенаправления: https://omgagi.ai/oauth/callback/\n\
             - *Скачайте JSON*\n\n\
             *4. Опубликовать приложение*\n\
             {consent_url}\n\
             - Перейдите в \"Audience\" и нажмите \"Publish App\"\n\n\
             *Вставьте полное содержимое скачанного JSON-файла, когда будете готовы.*"
        ),
        _ => format!(
            "Project received: *{project_id}*\n\n\
             Follow these steps:\n\n\
             *1. Enable APIs* (click each link and enable):\n\
             - Gmail: {gmail_url}\n\
             - Calendar: {calendar_url}\n\
             - Drive: {drive_url}\n\
             - Docs: {docs_url}\n\
             - Sheets: {sheets_url}\n\n\
             *2. OAuth consent screen*\n\
             {consent_url}\n\
             - Click \"Get Started\"\n\
             - Name: omega | Email: your email\n\
             - Audience: External | Create\n\n\
             *3. Create OAuth credentials*\n\
             {cred_url}\n\
             - Type: Web application\n\
             - Redirect URI: https://omgagi.ai/oauth/callback/\n\
             - *Download the JSON*\n\n\
             *4. Publish the app*\n\
             {consent_url}\n\
             - Go to \"Audience\" and click \"Publish App\"\n\n\
             *Paste the full content of the downloaded JSON file when ready.*"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_LANGUAGES: &[&str] = &[
        "English",
        "Spanish",
        "Portuguese",
        "French",
        "German",
        "Italian",
        "Dutch",
        "Russian",
    ];

    #[test]
    fn test_setup_guide_message_all_languages() {
        for lang in ALL_LANGUAGES {
            let msg = google_step_setup_guide_message(lang, "test-project");
            assert!(!msg.is_empty(), "setup_guide({lang}) must not be empty");
            assert!(
                msg.contains("test-project"),
                "setup_guide must contain project ID"
            );
        }
    }

    #[test]
    fn test_setup_guide_contains_api_links() {
        let msg = google_step_setup_guide_message("English", "my-proj-123");
        assert!(msg.contains("gmail.googleapis.com"));
        assert!(msg.contains("calendar-json.googleapis.com"));
        assert!(msg.contains("drive.googleapis.com"));
        assert!(msg.contains("my-proj-123"));
    }

    #[test]
    fn test_setup_guide_contains_console_links() {
        let msg = google_step_setup_guide_message("English", "my-proj");
        assert!(msg.contains("apis/credentials/consent"));
        assert!(msg.contains("apis/credentials/oauthclient"));
        assert!(msg.contains("omgagi.ai/oauth/callback"));
    }
}
