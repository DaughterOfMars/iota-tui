//! Portfolio summary aggregation across multiple accounts.

use crate::app::{App, PortfolioSummary};

use super::formatting::format_balance;

impl App {
    /// Compute portfolio summary: aggregate coins by type across all accounts.
    pub fn compute_portfolio_summary(&mut self) {
        use std::collections::BTreeMap;

        struct Agg {
            symbol: String,
            total: u128,
            per_account: Vec<(String, u128)>,
        }

        let mut by_type: BTreeMap<String, Agg> = BTreeMap::new();
        for coin in &self.coins {
            let entry = by_type
                .entry(coin.coin_type.clone())
                .or_insert_with(|| Agg {
                    symbol: coin.symbol.clone(),
                    total: 0,
                    per_account: vec![],
                });
            entry.total += coin.balance;
            if let Some(acct) = entry
                .per_account
                .iter_mut()
                .find(|(a, _)| *a == coin.owner_alias)
            {
                acct.1 += coin.balance;
            } else {
                entry
                    .per_account
                    .push((coin.owner_alias.clone(), coin.balance));
            }
        }
        self.portfolio_summary = by_type
            .into_iter()
            .map(|(coin_type, agg)| PortfolioSummary {
                coin_type,
                symbol: agg.symbol,
                total_balance_display: format_balance(agg.total, 9),
                per_account: agg
                    .per_account
                    .into_iter()
                    .map(|(alias, bal)| (alias, format_balance(bal, 9)))
                    .collect(),
            })
            .collect();
        if self.portfolio_selected >= self.portfolio_summary.len() {
            self.portfolio_selected = self.portfolio_summary.len().saturating_sub(1);
        }
    }
}
