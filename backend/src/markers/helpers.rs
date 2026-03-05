//! Miscellaneous helpers: status messages, provider errors, workspace images,
//! active hours, plan parsing, and inbox operations.

use std::path::PathBuf;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Status messages — random funny pools per language
// ---------------------------------------------------------------------------

/// Pick a pseudo-random index from subsecond nanos (good enough for fun phrases).
fn random_index(len: usize) -> usize {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
        % len
}

/// Return a random localized "first nudge" message (sent after 15 s of silence).
pub fn random_nudge_message(lang: &str) -> &'static str {
    let pool = match lang {
        "Spanish" => NUDGE_ES,
        "Portuguese" => NUDGE_PT,
        "French" => NUDGE_FR,
        "German" => NUDGE_DE,
        "Italian" => NUDGE_IT,
        "Dutch" => NUDGE_NL,
        "Russian" => NUDGE_RU,
        _ => NUDGE_EN,
    };
    pool[random_index(pool.len())]
}

/// Return a random localized "still working" message (sent every 120 s).
pub fn random_still_message(lang: &str) -> &'static str {
    let pool = match lang {
        "Spanish" => STILL_ES,
        "Portuguese" => STILL_PT,
        "French" => STILL_FR,
        "German" => STILL_DE,
        "Italian" => STILL_IT,
        "Dutch" => STILL_NL,
        "Russian" => STILL_RU,
        _ => STILL_EN,
    };
    pool[random_index(pool.len())]
}

/// All nudge message pools (for test validation).
#[cfg(test)]
pub fn nudge_pool(lang: &str) -> &'static [&'static str] {
    match lang {
        "Spanish" => NUDGE_ES,
        "Portuguese" => NUDGE_PT,
        "French" => NUDGE_FR,
        "German" => NUDGE_DE,
        "Italian" => NUDGE_IT,
        "Dutch" => NUDGE_NL,
        "Russian" => NUDGE_RU,
        _ => NUDGE_EN,
    }
}

/// All still-working message pools (for test validation).
#[cfg(test)]
pub fn still_pool(lang: &str) -> &'static [&'static str] {
    match lang {
        "Spanish" => STILL_ES,
        "Portuguese" => STILL_PT,
        "French" => STILL_FR,
        "German" => STILL_DE,
        "Italian" => STILL_IT,
        "Dutch" => STILL_NL,
        "Russian" => STILL_RU,
        _ => STILL_EN,
    }
}

// -- English ----------------------------------------------------------------

const NUDGE_EN: &[&str] = &[
    "Hmm, let me put my thinking cap on... 🎩",
    "Consulting the ancient scrolls... 📜",
    "My hamster wheel is spinning... 🐹",
    "Brewing something good... ☕",
    "Diving deep into the rabbit hole... 🕳️",
    "Hold tight, genius loading... 💡",
    "Summoning wisdom from the cloud... ☁️",
    "Crunching this like popcorn... 🍿",
    "Warming up the flux capacitor... ⚡",
    "Plot twist incoming... 🎬",
];

const STILL_EN: &[&str] = &[
    "Still cooking... almost crispy 🍳",
    "Rome wasn't built in a message 🏛️",
    "Good things take time... allegedly 🍀",
    "Still here, still fabulous 💅",
    "My other brain is a supercomputer 🖥️",
    "Worth the wait, I promise 🎁",
];

// -- Spanish ----------------------------------------------------------------

const NUDGE_ES: &[&str] = &[
    "Poniéndome el gorro de pensar... 🎩",
    "Consultando los pergaminos antiguos... 📜",
    "Mi hámster mental está corriendo... 🐹",
    "Preparando algo bueno... ☕",
    "Cayendo por la madriguera del conejo... 🕳️",
    "Calma, cargando genialidad... 💡",
    "Invocando sabiduría de la nube... ☁️",
    "Triturando esto como palomitas... 🍿",
    "Calentando el condensador de fluzo... ⚡",
    "Se viene plot twist... 🎬",
];

const STILL_ES: &[&str] = &[
    "Aún cocinando... casi crujiente 🍳",
    "Roma no se construyó en un mensaje 🏛️",
    "Lo bueno se hace esperar... dicen 🍀",
    "Sigo aquí, sigo fabuloso 💅",
    "Mi otro cerebro es un superordenador 🖥️",
    "Vale la espera, lo prometo 🎁",
];

// -- Portuguese -------------------------------------------------------------

const NUDGE_PT: &[&str] = &[
    "Colocando meu chapéu de pensar... 🎩",
    "Consultando os pergaminhos antigos... 📜",
    "Meu hamster mental tá correndo... 🐹",
    "Preparando algo bom... ☕",
    "Mergulhando na toca do coelho... 🕳️",
    "Calma, carregando genialidade... 💡",
    "Invocando sabedoria da nuvem... ☁️",
    "Mastigando isso como pipoca... 🍿",
    "Aquecendo o capacitor de fluxo... ⚡",
    "Vem plot twist por aí... 🎬",
];

const STILL_PT: &[&str] = &[
    "Ainda cozinhando... quase crocante 🍳",
    "Roma não foi construída numa mensagem 🏛️",
    "Coisas boas levam tempo... dizem 🍀",
    "Ainda aqui, ainda fabuloso 💅",
    "Meu outro cérebro é um supercomputador 🖥️",
    "Vale a espera, prometo 🎁",
];

// -- French -----------------------------------------------------------------

const NUDGE_FR: &[&str] = &[
    "Je mets mon chapeau de réflexion... 🎩",
    "Consultation des parchemins anciens... 📜",
    "Mon hamster cérébral court à fond... 🐹",
    "Je prépare un truc bien... ☕",
    "Plongée dans le terrier du lapin... 🕳️",
    "Du calme, génie en chargement... 💡",
    "J'invoque la sagesse du cloud... ☁️",
    "Je croque ça comme du popcorn... 🍿",
    "Préchauffage du convecteur temporel... ⚡",
    "Rebondissement en approche... 🎬",
];

const STILL_FR: &[&str] = &[
    "Ça mijote encore... presque croustillant 🍳",
    "Rome ne s'est pas faite en un message 🏛️",
    "Les bonnes choses prennent du temps... paraît-il 🍀",
    "Toujours là, toujours fabuleux 💅",
    "Mon autre cerveau est un supercalculateur 🖥️",
    "Ça vaut l'attente, promis 🎁",
];

// -- German -----------------------------------------------------------------

const NUDGE_DE: &[&str] = &[
    "Setze meine Denkmütze auf... 🎩",
    "Konsultiere die alten Schriftrollen... 📜",
    "Mein Denkhamster rennt auf Hochtouren... 🐹",
    "Braue etwas Gutes zusammen... ☕",
    "Tauche tief in den Kaninchenbau... 🕳️",
    "Ruhe bitte, Genie lädt... 💡",
    "Beschwöre Weisheit aus der Cloud... ☁️",
    "Knacke das wie Popcorn... 🍿",
    "Heize den Fluxkompensator vor... ⚡",
    "Plot-Twist im Anmarsch... 🎬",
];

const STILL_DE: &[&str] = &[
    "Köchelt noch... fast knusprig 🍳",
    "Rom wurde nicht in einer Nachricht erbaut 🏛️",
    "Gut Ding will Weile haben... angeblich 🍀",
    "Immer noch hier, immer noch fabelhaft 💅",
    "Mein zweites Gehirn ist ein Supercomputer 🖥️",
    "Das Warten lohnt sich, versprochen 🎁",
];

// -- Italian ----------------------------------------------------------------

const NUDGE_IT: &[&str] = &[
    "Mi metto il cappello da pensatore... 🎩",
    "Consulto le pergamene antiche... 📜",
    "Il mio criceto mentale corre a tutta... 🐹",
    "Sto preparando qualcosa di buono... ☕",
    "Mi tuffo nella tana del coniglio... 🕳️",
    "Calma, genio in caricamento... 💡",
    "Evoco saggezza dal cloud... ☁️",
    "Sgranocchio questo come popcorn... 🍿",
    "Riscaldo il flusso canalizzatore... ⚡",
    "Colpo di scena in arrivo... 🎬",
];

const STILL_IT: &[&str] = &[
    "Sta ancora cuocendo... quasi croccante 🍳",
    "Roma non fu costruita in un messaggio 🏛️",
    "Le cose belle richiedono tempo... dicono 🍀",
    "Ancora qui, ancora favoloso 💅",
    "Il mio altro cervello è un supercomputer 🖥️",
    "Vale l'attesa, promesso 🎁",
];

// -- Dutch ------------------------------------------------------------------

const NUDGE_NL: &[&str] = &[
    "Even mijn denkhoed opzetten... 🎩",
    "De oude geschriften raadplegen... 📜",
    "Mijn denkhamster draait overuren... 🐹",
    "Iets goeds aan het brouwen... ☕",
    "Duik in het konijnenhol... 🕳️",
    "Rustig, genie aan het laden... 💡",
    "Wijsheid oproepen uit de cloud... ☁️",
    "Dit kraken als popcorn... 🍿",
    "De fluxcondensator opwarmen... ⚡",
    "Plot twist in aantocht... 🎬",
];

const STILL_NL: &[&str] = &[
    "Nog aan het koken... bijna knapperig 🍳",
    "Rome werd niet in één bericht gebouwd 🏛️",
    "Goeie dingen kosten tijd... schijnt 🍀",
    "Nog hier, nog steeds fantastisch 💅",
    "Mijn andere brein is een supercomputer 🖥️",
    "Het wachten waard, beloofd 🎁",
];

// -- Russian ----------------------------------------------------------------

const NUDGE_RU: &[&str] = &[
    "Надеваю шапку мыслителя... 🎩",
    "Сверяюсь с древними свитками... 📜",
    "Мой мысленный хомяк бежит изо всех сил... 🐹",
    "Готовлю что-то хорошее... ☕",
    "Ныряю в кроличью нору... 🕳️",
    "Спокойно, гениальность загружается... 💡",
    "Призываю мудрость из облака... ☁️",
    "Щёлкаю это как попкорн... 🍿",
    "Прогреваю потоковый конденсатор... ⚡",
    "Сюжетный поворот на подходе... 🎬",
];

const STILL_RU: &[&str] = &[
    "Всё ещё готовлю... почти хрустит 🍳",
    "Рим не в одном сообщении построили 🏛️",
    "Хорошее требует времени... говорят 🍀",
    "Всё ещё тут, всё ещё великолепен 💅",
    "Мой второй мозг — суперкомпьютер 🖥️",
    "Ожидание того стоит, обещаю 🎁",
];

/// Map raw provider errors to user-friendly messages.
pub fn friendly_provider_error(raw: &str) -> String {
    if raw.contains("timed out") {
        "I took too long to respond. Please try again — sometimes complex requests need a second attempt.".to_string()
    } else {
        "Something went wrong. Please try again.".to_string()
    }
}

// ---------------------------------------------------------------------------
// Workspace images
// ---------------------------------------------------------------------------

/// Image file extensions recognized for workspace diff.
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// Snapshot top-level image files in the workspace directory.
///
/// Returns a map of path → modification time. Returns an empty map on any
/// error (non-existent dir, permission issues). Tracks mtime so we can detect
/// both new files and overwritten files (same name, newer mtime).
pub fn snapshot_workspace_images(
    workspace: &std::path::Path,
) -> std::collections::HashMap<PathBuf, std::time::SystemTime> {
    let entries = match std::fs::read_dir(workspace) {
        Ok(e) => e,
        Err(_) => return std::collections::HashMap::new(),
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                && entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
        })
        .filter_map(|entry| {
            let mtime = entry.metadata().ok()?.modified().ok()?;
            Some((entry.path(), mtime))
        })
        .collect()
}

/// Check if the current local time is within the active hours window.
pub fn is_within_active_hours(start: &str, end: &str) -> bool {
    let now = chrono::Local::now().format("%H:%M").to_string();
    if start <= end {
        // Normal range: e.g. 08:00 to 22:00
        now.as_str() >= start && now.as_str() < end
    } else {
        // Midnight wrap: e.g. 22:00 to 06:00
        now.as_str() >= start || now.as_str() < end
    }
}

/// Compute the next occurrence of `active_start` (local "HH:MM") as a UTC
/// datetime string suitable for `scheduled_tasks.due_at`.
pub fn next_active_start_utc(start: &str) -> String {
    use chrono::{Local, NaiveTime, TimeZone};

    let now = Local::now();
    let start_time = NaiveTime::parse_from_str(start, "%H:%M")
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(8, 0, 0).expect("08:00 is always valid"));

    let today_candidate = now.date_naive().and_time(start_time);
    let candidate = if today_candidate > now.naive_local() {
        today_candidate
    } else {
        today_candidate + chrono::Duration::days(1)
    };

    // Convert local candidate to UTC for the DB.
    let local_dt = Local
        .from_local_datetime(&candidate)
        .earliest()
        .unwrap_or(now);
    local_dt
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Parse a plan/classification response into numbered steps.
///
/// Returns `None` if the response is "DIRECT" (case-insensitive) or has fewer
/// than 2 steps. Steps are extracted from lines starting with `N.` where N is
/// a digit.
pub fn parse_plan_response(text: &str) -> Option<Vec<String>> {
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("DIRECT") {
        return None;
    }

    let steps: Vec<String> = trimmed
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Match lines starting with "N. " where N is a digit.
            if line.len() >= 3 && line.as_bytes()[0].is_ascii_digit() && line.as_bytes()[1] == b'.'
            {
                Some(line[2..].trim().to_string())
            } else {
                None
            }
        })
        .collect();

    if steps.len() >= 2 {
        Some(steps)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Inbox helpers
// ---------------------------------------------------------------------------

/// Ensure the workspace inbox directory exists and return its path.
pub fn ensure_inbox_dir(data_dir: &str) -> PathBuf {
    let dir = PathBuf::from(omega_core::config::shellexpand(data_dir))
        .join("workspace")
        .join("inbox");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Save image attachments to the inbox directory and return the paths.
///
/// Rejects zero-byte attachments and uses `sync_all` to guarantee
/// the data hits disk before the path is returned.
pub fn save_attachments_to_inbox(
    inbox: &std::path::Path,
    attachments: &[omega_core::message::Attachment],
) -> Vec<PathBuf> {
    use std::io::Write;

    let mut paths = Vec::new();
    for attachment in attachments {
        if !matches!(
            attachment.file_type,
            omega_core::message::AttachmentType::Image
        ) {
            continue;
        }
        if let Some(ref data) = attachment.data {
            if data.is_empty() {
                tracing::warn!("skipping zero-byte image attachment");
                continue;
            }
            let filename = attachment
                .filename
                .as_deref()
                .unwrap_or("image.jpg")
                .to_string();
            let path = inbox.join(&filename);
            match std::fs::File::create(&path) {
                Ok(mut file) => {
                    if file.write_all(data).is_ok() && file.sync_all().is_ok() {
                        tracing::debug!("inbox: wrote {} ({} bytes)", path.display(), data.len());
                        paths.push(path);
                    } else {
                        tracing::warn!("inbox: failed to write {}", path.display());
                    }
                }
                Err(e) => {
                    tracing::warn!("inbox: failed to create {}: {e}", path.display());
                }
            }
        }
    }
    paths
}

/// RAII guard that cleans up inbox image files when dropped.
///
/// Guarantees cleanup regardless of early returns in `handle_message()`.
pub struct InboxGuard {
    paths: Vec<PathBuf>,
}

impl InboxGuard {
    /// Create a new guard that will clean up the given paths on drop.
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }
}

impl Drop for InboxGuard {
    fn drop(&mut self) {
        cleanup_inbox_images(&self.paths);
    }
}

/// Delete inbox images after they have been processed.
pub fn cleanup_inbox_images(paths: &[PathBuf]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

/// Purge all files in the inbox directory (startup cleanup).
pub fn purge_inbox(data_dir: &str) {
    let inbox = ensure_inbox_dir(data_dir);
    if let Ok(entries) = std::fs::read_dir(&inbox) {
        let mut count = 0u32;
        for entry in entries.flatten() {
            if entry.path().is_file() {
                let _ = std::fs::remove_file(entry.path());
                count += 1;
            }
        }
        if count > 0 {
            tracing::info!("startup: purged {count} orphaned inbox file(s)");
        }
    }
}
