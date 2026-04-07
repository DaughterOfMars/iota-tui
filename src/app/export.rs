//! CSV export functionality.

use crate::app::App;

impl App {
    /// Export current screen data to a CSV file.
    pub fn export_csv(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let screen_name = self.screen.title().to_lowercase().replace(' ', "-");
        let filename = format!("iota-export-{}-{}.csv", screen_name, timestamp);
        let path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(&filename);

        let content = match self.screen {
            crate::app::Screen::Coins => {
                let mut csv = "Symbol,Type,Balance,Object ID,Owner\n".to_string();
                for c in &self.coins {
                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        c.symbol, c.coin_type, c.balance_display, c.object_id, c.owner_alias
                    ));
                }
                csv
            }
            crate::app::Screen::Objects => {
                let mut csv = "Object ID,Type,Version,Digest,Owner\n".to_string();
                for o in &self.objects {
                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        o.object_id, o.type_name, o.version, o.digest, o.owner_alias
                    ));
                }
                csv
            }
            crate::app::Screen::Transactions => {
                let mut csv = "Digest,Status,Gas Used,Epoch\n".to_string();
                for t in &self.transactions {
                    csv.push_str(&format!(
                        "{},{},{},{}\n",
                        t.digest, t.status, t.gas_used, t.epoch
                    ));
                }
                csv
            }
            crate::app::Screen::Staking => {
                let mut csv = "Object ID,Principal,Validator,Epoch,Status\n".to_string();
                for s in &self.stakes {
                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        s.object_id,
                        s.principal_display,
                        s.validator_address,
                        s.activation_epoch,
                        s.status
                    ));
                }
                csv
            }
            crate::app::Screen::Keys => {
                let mut csv = "Alias,Address,Scheme,Active\n".to_string();
                for k in &self.keys {
                    csv.push_str(&format!(
                        "{},{},{},{}\n",
                        k.alias, k.address, k.scheme, k.is_active
                    ));
                }
                csv
            }
            crate::app::Screen::AddressBook => {
                let mut csv = "Label,Address,Notes\n".to_string();
                for e in &self.address_book {
                    csv.push_str(&format!("{},{},{}\n", e.label, e.address, e.notes));
                }
                csv
            }
            _ => {
                self.clipboard_toast = Some((
                    "Export not available for this screen".into(),
                    std::time::Instant::now(),
                ));
                return;
            }
        };

        match std::fs::write(&path, content) {
            Ok(()) => {
                self.clipboard_toast = Some((
                    format!("Exported to ~/{}", filename),
                    std::time::Instant::now(),
                ));
            }
            Err(e) => {
                self.clipboard_toast =
                    Some((format!("Export failed: {}", e), std::time::Instant::now()));
            }
        }
    }
}
