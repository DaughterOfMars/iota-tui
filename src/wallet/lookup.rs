//! Lookup and address-page handler methods for `WalletBackend`.

use iota_sdk::graphql_client::{Client, Direction, PaginationFilter, query_types::ObjectFilter};
use iota_sdk::types::ObjectType;

use super::WalletBackend;
use super::WalletEvent;
use super::helpers::{
    build_tx_sections_v1, format_gas, format_json_value, format_owner, guess_action_from_value,
    owner_action, prettify_struct,
};

impl WalletBackend {
    pub(super) async fn handle_lookup(
        &self,
        query: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::app::{LookupAction, LookupField, LookupResult, LookupSection};
        let client = self.client.as_ref().ok_or("Not connected")?;

        let hex_query = if query.starts_with("0x") {
            query.to_string()
        } else {
            format!("0x{}", query)
        };

        // Try as object first
        if let Ok(addr) = iota_sdk::types::Address::from_hex(&hex_query) {
            let obj_id: iota_sdk::types::ObjectId = addr.into();
            if let Ok(Some(obj)) = client.object(obj_id, None).await {
                let type_name = match obj.object_type() {
                    ObjectType::Struct(s) => prettify_struct(&s),
                    ObjectType::Package => "Package".into(),
                };
                let type_raw = match obj.object_type() {
                    ObjectType::Struct(s) => s.to_string(),
                    ObjectType::Package => "Package".into(),
                };
                let owner_str = format_owner(&obj.owner);

                let info_fields = vec![
                    LookupField {
                        key: "Object ID".into(),
                        value: obj.object_id().to_string(),
                        action: None,
                    },
                    LookupField {
                        key: "Version".into(),
                        value: format!("v{}", obj.version()),
                        action: None,
                    },
                    LookupField {
                        key: "Type".into(),
                        value: type_name,
                        action: Some(LookupAction::TypeSearch(type_raw)),
                    },
                    LookupField {
                        key: "Owner".into(),
                        value: owner_str.clone(),
                        action: owner_action(&obj.owner),
                    },
                    LookupField {
                        key: "Previous Tx".into(),
                        value: obj.previous_transaction.to_string(),
                        action: Some(LookupAction::Explore(obj.previous_transaction.to_string())),
                    },
                ];

                let mut sections = vec![LookupSection {
                    title: "Object".into(),
                    fields: info_fields,
                }];

                // Fetch move object content (JSON fields) into a separate section
                if let Ok(Some(json)) = client.move_object_contents(obj_id, None).await
                    && let Some(map) = json.as_object()
                {
                    let field_entries: Vec<LookupField> = map
                        .iter()
                        .map(|(k, v)| {
                            let val_str = format_json_value(v);
                            let action = guess_action_from_value(&val_str);
                            LookupField {
                                key: k.clone(),
                                value: val_str,
                                action,
                            }
                        })
                        .collect();
                    if !field_entries.is_empty() {
                        sections.push(LookupSection {
                            title: format!("Fields ({})", field_entries.len()),
                            fields: field_entries,
                        });
                    }
                }

                // Fetch dynamic fields
                let df_page = client
                    .dynamic_fields(addr, PaginationFilter::default())
                    .await;
                if let Ok(df_page) = df_page {
                    let dfs = df_page.data();
                    if !dfs.is_empty() {
                        let mut df_fields: Vec<LookupField> = Vec::new();
                        for df in dfs {
                            let name_str = df
                                .name
                                .json
                                .as_ref()
                                .map(format_json_value)
                                .unwrap_or_else(|| format!("{}", df.name.type_));
                            let val_str = df
                                .value_as_json
                                .as_ref()
                                .map(format_json_value)
                                .unwrap_or_else(|| "?".into());
                            let action = guess_action_from_value(&val_str);
                            df_fields.push(LookupField {
                                key: name_str,
                                value: val_str,
                                action,
                            });
                        }
                        sections.push(LookupSection {
                            title: format!("Dynamic Fields ({})", df_fields.len()),
                            fields: df_fields,
                        });
                    }
                }

                self.event_tx
                    .send(WalletEvent::ExplorerLookupResult(LookupResult::Object {
                        sections,
                    }))
                    .await?;
                return Ok(());
            }
        }

        // Try as transaction digest
        if let Ok(digest) = hex_query
            .parse::<iota_sdk::types::Digest>()
            .or_else(|_| query.parse::<iota_sdk::types::Digest>())
            && let Ok(Some(td)) = client.transaction_data_effects(digest).await
        {
            let sections = match &td.effects {
                iota_sdk::types::TransactionEffects::V1(v1) => build_tx_sections_v1(v1, &td.tx),
                _ => vec![LookupSection {
                    title: "Transaction".into(),
                    fields: vec![LookupField {
                        key: "Note".into(),
                        value: "Unsupported transaction effects version".into(),
                        action: None,
                    }],
                }],
            };

            self.event_tx
                .send(WalletEvent::ExplorerLookupResult(
                    LookupResult::Transaction { sections },
                ))
                .await?;
            return Ok(());
        }

        // Try as address (look up owned objects + balance + transactions)
        if let Ok(addr) = iota_sdk::types::Address::from_hex(&hex_query) {
            let balance = client.balance(addr, None).await.unwrap_or(None);

            let obj_page = client
                .objects(
                    ObjectFilter {
                        owner: Some(addr),
                        type_: None,
                        object_ids: None,
                    },
                    PaginationFilter {
                        direction: Direction::Backward,
                        cursor: None,
                        limit: Some(20),
                    },
                )
                .await?;

            let has_data = !obj_page.data().is_empty() || balance.is_some();

            if has_data {
                let (sections, obj_cursor, obj_has_next, tx_cursor, tx_has_next) =
                    Self::build_address_sections(client, &hex_query, addr, balance, obj_page, None)
                        .await;

                self.event_tx
                    .send(WalletEvent::AddressLookupPage {
                        result: LookupResult::Address { sections },
                        obj_cursor,
                        obj_has_next,
                        tx_cursor,
                        tx_has_next,
                    })
                    .await?;
                return Ok(());
            }
        }

        self.event_tx
            .send(WalletEvent::ExplorerLookupResult(LookupResult::NotFound(
                format!("Nothing found for '{}'", query),
            )))
            .await?;
        Ok(())
    }

    pub(super) async fn build_address_sections(
        client: &Client,
        hex_query: &str,
        addr: iota_sdk::types::Address,
        balance: Option<u64>,
        obj_page: iota_sdk::graphql_client::Page<iota_sdk::types::Object>,
        tx_cursor: Option<String>,
    ) -> (
        Vec<crate::app::LookupSection>,
        Option<String>,
        bool,
        Option<String>,
        bool,
    ) {
        use crate::app::{LookupAction, LookupField, LookupSection};

        let balance_str = balance.map(format_gas).unwrap_or_else(|| "0".into());
        let overview = vec![
            LookupField {
                key: "Address".into(),
                value: hex_query.to_string(),
                action: None,
            },
            LookupField {
                key: "IOTA Balance".into(),
                value: balance_str,
                action: None,
            },
        ];
        let mut sections = vec![LookupSection {
            title: "Address".into(),
            fields: overview,
        }];

        // Objects section
        let obj_cursor_out = obj_page.page_info().start_cursor.clone();
        let obj_has_next = obj_page.page_info().has_previous_page;
        if !obj_page.data().is_empty() {
            let obj_fields: Vec<LookupField> = obj_page
                .data()
                .iter()
                .enumerate()
                .map(|(i, obj)| {
                    let id_str = obj.object_id().to_string();
                    LookupField {
                        key: format!("{}", i),
                        value: id_str.clone(),
                        action: Some(LookupAction::Explore(id_str)),
                    }
                })
                .collect();
            sections.push(LookupSection {
                title: format!("Objects ({})", obj_fields.len()),
                fields: obj_fields,
            });
        }

        // Transactions section
        let mut tx_cursor_out: Option<String> = None;
        let mut tx_has_next = false;
        {
            use iota_sdk::graphql_client::query_types::TransactionsFilter;
            let tx_filter = TransactionsFilter {
                sign_address: Some(addr),
                ..Default::default()
            };
            if let Ok(tx_page) = client
                .transactions_effects(
                    tx_filter,
                    PaginationFilter {
                        direction: Direction::Backward,
                        cursor: tx_cursor,
                        limit: Some(20),
                    },
                )
                .await
            {
                tx_cursor_out = tx_page.page_info().start_cursor.clone();
                tx_has_next = tx_page.page_info().has_previous_page;
                if !tx_page.data().is_empty() {
                    let tx_fields: Vec<LookupField> = tx_page
                        .data()
                        .iter()
                        .map(|effects| match effects {
                            iota_sdk::types::TransactionEffects::V1(ev1) => {
                                let status = match &ev1.status {
                                    iota_sdk::types::ExecutionStatus::Success => "OK",
                                    _ => "FAIL",
                                };
                                let digest_str = ev1.transaction_digest.to_string();
                                LookupField {
                                    key: status.into(),
                                    value: digest_str.clone(),
                                    action: Some(LookupAction::Explore(digest_str)),
                                }
                            }
                            _ => LookupField {
                                key: "?".into(),
                                value: "Unsupported effects version".into(),
                                action: None,
                            },
                        })
                        .collect();
                    sections.push(LookupSection {
                        title: format!("Transactions ({})", tx_fields.len()),
                        fields: tx_fields,
                    });
                }
            }
        }

        (
            sections,
            obj_cursor_out,
            obj_has_next,
            tx_cursor_out,
            tx_has_next,
        )
    }

    pub(super) async fn handle_address_page(
        &self,
        address: &str,
        obj_cursor: Option<String>,
        tx_cursor: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::app::LookupResult;
        let client = self.client.as_ref().ok_or("Not connected")?;
        let hex_addr = if address.starts_with("0x") {
            address.to_string()
        } else {
            format!("0x{}", address)
        };
        let addr = iota_sdk::types::Address::from_hex(&hex_addr)?;

        let balance = client.balance(addr, None).await.unwrap_or(None);
        let obj_page = client
            .objects(
                ObjectFilter {
                    owner: Some(addr),
                    type_: None,
                    object_ids: None,
                },
                PaginationFilter {
                    direction: Direction::Backward,
                    cursor: obj_cursor,
                    limit: Some(20),
                },
            )
            .await?;

        let (sections, obj_cursor_out, obj_has_next, tx_cursor_out, tx_has_next) =
            Self::build_address_sections(client, &hex_addr, addr, balance, obj_page, tx_cursor)
                .await;

        self.event_tx
            .send(WalletEvent::AddressLookupPage {
                result: LookupResult::Address { sections },
                obj_cursor: obj_cursor_out,
                obj_has_next,
                tx_cursor: tx_cursor_out,
                tx_has_next,
            })
            .await?;
        Ok(())
    }
}
