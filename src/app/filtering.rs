//! Filtered list queries for coins, objects, transactions, and packages.

use crate::app::App;

impl App {
    /// Returns indices into `self.objects` for objects that look like packages.
    pub fn package_indices(&self) -> Vec<usize> {
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.type_name.contains("package")
                    || o.type_name == "Package"
                    || o.type_name.is_empty()
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Returns indices into `self.coins` matching the current filter.
    pub fn filtered_coins(&self) -> Vec<usize> {
        let Some(ref q) = self.coins_filter else {
            return (0..self.coins.len()).collect();
        };
        if q.is_empty() {
            return (0..self.coins.len()).collect();
        }
        let q = q.to_lowercase();
        self.coins
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.symbol.to_lowercase().contains(&q)
                    || c.coin_type.to_lowercase().contains(&q)
                    || c.balance_display.contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Returns indices into `self.objects` matching the current filter.
    pub fn filtered_objects(&self) -> Vec<usize> {
        let Some(ref q) = self.objects_filter else {
            return (0..self.objects.len()).collect();
        };
        if q.is_empty() {
            return (0..self.objects.len()).collect();
        }
        let q = q.to_lowercase();
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.type_name.to_lowercase().contains(&q) || o.object_id.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Returns indices into `self.transactions` matching the current filter.
    pub fn filtered_transactions(&self) -> Vec<usize> {
        let Some(ref q) = self.transactions_filter else {
            return (0..self.transactions.len()).collect();
        };
        if q.is_empty() {
            return (0..self.transactions.len()).collect();
        }
        let q = q.to_lowercase();
        self.transactions
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.digest.to_lowercase().contains(&q)
                    || t.status.to_lowercase().contains(&q)
                    || t.epoch.contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }
}
