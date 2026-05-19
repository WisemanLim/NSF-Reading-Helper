//! OS-level TTS shim. Web build goes through window.speechSynthesis directly;
//! desktop falls through to platform tools (macOS `say`, Linux `espeak-ng`,
//! Windows PowerShell). Best-effort — failures are non-fatal.

use crate::core::error::AppResult;
use crate::core::models::TtsOptions;
use tokio::process::Command;

pub async fn speak(text: &str, opts: &TtsOptions) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let rate_wpm = (180.0 * opts.rate).clamp(80.0, 400.0) as i64;
        let _ = Command::new("say")
            .args(["-r", &rate_wpm.to_string(), text])
            .spawn()
            .map(|mut c| async move {
                let _ = c.wait().await;
            });
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        let _lang = opts.lang.as_str();
        let _ = Command::new("espeak-ng")
            .args([
                "-v",
                if opts.lang.starts_with("ko") { "ko" } else { "en" },
                "-s",
                &(160.0 * opts.rate).clamp(80.0, 400.0).to_string(),
                text,
            ])
            .spawn();
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        let ps = format!(
            "Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
            text.replace('\'', "''")
        );
        let _ = Command::new("powershell").args(["-Command", &ps]).spawn();
        let _ = opts.rate;
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = (text, opts);
        Ok(())
    }
}

pub async fn stop() -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("killall").arg("say").spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("killall").arg("espeak-ng").spawn();
    }
    Ok(())
}
