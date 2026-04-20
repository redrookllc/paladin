use crate::ui;
use paladin_core::error::Result;

pub async fn run(_update_self: bool) -> Result<()> {
    println!(
        "  {} self-update is a stub in this build; run `cargo install --path crates/paladin-cli` to refresh.",
        ui::warn_glyph()
    );
    Ok(())
}
