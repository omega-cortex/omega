//! Standalone WhatsApp pairing via QR code.

/// Run the interactive WhatsApp pairing flow.
pub async fn pair_whatsapp() -> anyhow::Result<()> {
    use omega_channels::whatsapp;
    use omega_core::shellexpand;
    use std::path::Path;

    cliclack::intro(console::style("omega pair").bold().to_string())?;

    let session_db = shellexpand("~/.omega/whatsapp_session/whatsapp.db");
    let already_paired = Path::new(&session_db).exists();

    if already_paired {
        cliclack::log::success("WhatsApp is already paired.")?;
        let reprovision: bool = cliclack::confirm("Re-pair? This will unlink the current session.")
            .initial_value(false)
            .interact()?;
        if !reprovision {
            cliclack::outro("Nothing changed.")?;
            return Ok(());
        }
        // Delete stale session so the library generates a fresh QR.
        let session_dir = shellexpand("~/.omega/whatsapp_session");
        let _ = std::fs::remove_dir_all(&session_dir);
        cliclack::log::step("Old session removed.")?;
    }

    cliclack::log::info("Open WhatsApp on your phone → Linked Devices → Link a Device")?;

    let (mut qr_rx, mut done_rx) = whatsapp::start_pairing("~/.omega").await?;

    // Wait for first QR code.
    let qr_data = tokio::time::timeout(std::time::Duration::from_secs(30), qr_rx.recv())
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for QR code"))?
        .ok_or_else(|| anyhow::anyhow!("QR channel closed"))?;

    let qr_text = whatsapp::generate_qr_terminal(&qr_data)?;
    cliclack::note("Scan this QR code with WhatsApp", &qr_text)?;

    let spinner = cliclack::spinner();
    spinner.start("Waiting for scan...");

    let paired = tokio::time::timeout(std::time::Duration::from_secs(60), done_rx.recv())
        .await
        .map_err(|_| anyhow::anyhow!("pairing timed out"))?
        .unwrap_or(false);

    if paired {
        spinner.stop("WhatsApp linked successfully!");
        cliclack::outro("Pairing complete. Restart omega to pick up the new session.")?;
    } else {
        spinner.error("Pairing did not complete.");
        cliclack::outro("Try again with: omega pair")?;
    }

    Ok(())
}
