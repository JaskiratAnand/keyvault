use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct PasswordSnapshot {
    pub password: String,
    pub changed_at: String, // ISO 8601 string timestamps
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String, // String representation of a UUID
    pub username: String,
    pub password: String,
    pub totp_secret: Option<String>,
    pub notes: String,
    pub custom_fields: Vec<CustomField>,
    pub password_history: Vec<PasswordSnapshot>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct DomainGroup {
    pub id: String, // String representation of a UUID
    pub title: String,
    pub urls: Vec<String>,
    pub accounts: Vec<Account>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl DomainGroup {
    pub fn merge(&mut self, other: &DomainGroup) {
        // Merge accounts using LWW
        for other_acc in &other.accounts {
            if let Some(self_acc) = self.accounts.iter_mut().find(|a| a.id == other_acc.id) {
                if other_acc.updated_at > self_acc.updated_at {
                    *self_acc = other_acc.clone();
                }
            } else {
                self.accounts.push(other_acc.clone());
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SecureNote {
    pub id: String, // String representation of a UUID
    pub title: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VaultItem {
    DomainGroup(DomainGroup),
    SecureNote(SecureNote),
}

impl VaultItem {
    pub fn id(&self) -> &str {
        match self {
            VaultItem::DomainGroup(dg) => &dg.id,
            VaultItem::SecureNote(sn) => &sn.id,
        }
    }

    pub fn updated_at(&self) -> &str {
        match self {
            VaultItem::DomainGroup(dg) => &dg.updated_at,
            VaultItem::SecureNote(sn) => &sn.updated_at,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub id: String,
    pub deleted_at: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Vault {
    #[serde(default)]
    pub items: Vec<VaultItem>,
    #[serde(default)]
    pub tombstones: Vec<Tombstone>,
    #[serde(default)]
    pub trash: Vec<VaultItem>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        Vault {
            items: Vec::new(),
            tombstones: Vec::new(),
            trash: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: VaultItem) {
        self.items.push(item);
    }

    pub fn update_item(&mut self, updated: VaultItem) -> Result<(), &str> {
        let id = updated.id().to_string();
        if let Some(item) = self.items.iter_mut().find(|e| e.id() == id) {
            *item = updated;
            Ok(())
        } else {
            Err("Item not found")
        }
    }

    pub fn delete_item(&mut self, id: &str, deleted_at: &str) -> Result<(), &str> {
        if self.trash.iter().any(|e| e.id() == id) {
            return Ok(());
        }
        if let Some(pos) = self.items.iter().position(|e| e.id() == id) {
            let mut item = self.items.remove(pos);
            match &mut item {
                VaultItem::DomainGroup(dg) => dg.updated_at = deleted_at.to_string(),
                VaultItem::SecureNote(sn) => sn.updated_at = deleted_at.to_string(),
            }
            self.trash.push(item);
            Ok(())
        } else {
            Err("Item not found")
        }
    }

    pub fn restore_item(&mut self, id: &str, restored_at: &str) -> Result<(), &str> {
        if self.items.iter().any(|e| e.id() == id) {
            return Ok(());
        }
        if let Some(pos) = self.trash.iter().position(|e| e.id() == id) {
            let mut item = self.trash.remove(pos);
            match &mut item {
                VaultItem::DomainGroup(dg) => dg.updated_at = restored_at.to_string(),
                VaultItem::SecureNote(sn) => sn.updated_at = restored_at.to_string(),
            }
            self.items.push(item);
            Ok(())
        } else {
            Err("Item not found in trash")
        }
    }

    pub fn purge_item(&mut self, id: &str, purged_at: &str) -> Result<(), &str> {
        let in_items = self.items.iter().any(|e| e.id() == id);
        let in_trash = self.trash.iter().any(|e| e.id() == id);

        // Also check if it's an account ID inside any of the domain groups
        let mut account_purged = false;
        for item in &mut self.items {
            if let VaultItem::DomainGroup(dg) = item
                && let Some(pos) = dg.accounts.iter().position(|a| a.id == id) {
                    dg.accounts.remove(pos);
                    dg.updated_at = purged_at.to_string();
                    account_purged = true;
                    break;
                }
        }
        for item in &mut self.trash {
            if let VaultItem::DomainGroup(dg) = item
                && let Some(pos) = dg.accounts.iter().position(|a| a.id == id) {
                    dg.accounts.remove(pos);
                    dg.updated_at = purged_at.to_string();
                    account_purged = true;
                    break;
                }
        }

        if !in_items && !in_trash && !account_purged {
            if self.tombstones.iter().any(|t| t.id == id) {
                return Ok(());
            }
            self.tombstones.push(Tombstone {
                id: id.to_string(),
                deleted_at: purged_at.to_string(),
            });
            return Ok(());
        }

        if in_items || in_trash {
            self.items.retain(|e| e.id() != id);
            self.trash.retain(|e| e.id() != id);
        }

        if !self.tombstones.iter().any(|t| t.id == id) {
            self.tombstones.push(Tombstone {
                id: id.to_string(),
                deleted_at: purged_at.to_string(),
            });
        }

        Ok(())
    }

    pub fn merge(&mut self, other: Vault) {
        // Merge tombstones lists
        for other_t in other.tombstones {
            if let Some(self_t) = self.tombstones.iter_mut().find(|t| t.id == other_t.id) {
                if other_t.deleted_at > self_t.deleted_at {
                    self_t.deleted_at = other_t.deleted_at;
                }
            } else {
                self.tombstones.push(other_t);
            }
        }

        // Merge items lists (LWW)
        for other_item in other.items {
            if let Some(self_item) = self.items.iter_mut().find(|e| e.id() == other_item.id()) {
                if other_item.updated_at() > self_item.updated_at() {
                    *self_item = other_item;
                } else if other_item.updated_at() == self_item.updated_at()
                    && let (VaultItem::DomainGroup(self_dg), VaultItem::DomainGroup(other_dg)) =
                        (self_item, &other_item)
                    {
                        self_dg.merge(other_dg);
                    }
            } else {
                self.items.push(other_item);
            }
        }

        // Merge trash lists (LWW)
        for other_trash in other.trash {
            if let Some(self_trash) = self.trash.iter_mut().find(|e| e.id() == other_trash.id()) {
                if other_trash.updated_at() > self_trash.updated_at() {
                    *self_trash = other_trash;
                }
            } else {
                self.trash.push(other_trash);
            }
        }

        // If an item is in both items and trash, keep only the latest one
        let mut to_remove_from_items = Vec::new();
        let mut to_remove_from_trash = Vec::new();

        for item in &self.items {
            if let Some(trash_item) = self.trash.iter().find(|t| t.id() == item.id()) {
                if item.updated_at() >= trash_item.updated_at() {
                    to_remove_from_trash.push(item.id().to_string());
                } else {
                    to_remove_from_items.push(item.id().to_string());
                }
            }
        }

        self.items
            .retain(|e| !to_remove_from_items.contains(&e.id().to_string()));
        self.trash
            .retain(|t| !to_remove_from_trash.contains(&t.id().to_string()));

        // Filter out any items or accounts that are tombed
        let tombstones = &self.tombstones;
        self.items.retain(|item| {
            if let Some(tombstone) = tombstones.iter().find(|t| t.id == item.id()) {
                item.updated_at() > &tombstone.deleted_at
            } else {
                true
            }
        });

        self.trash.retain(|item| {
            if let Some(tombstone) = tombstones.iter().find(|t| t.id == item.id()) {
                item.updated_at() > &tombstone.deleted_at
            } else {
                true
            }
        });

        // Filter out any accounts inside DomainGroups that have tombstones
        for item in &mut self.items {
            if let VaultItem::DomainGroup(dg) = item {
                dg.accounts.retain(|account| {
                    if let Some(tombstone) = tombstones.iter().find(|t| t.id == account.id) {
                        account.updated_at > tombstone.deleted_at
                    } else {
                        true
                    }
                });
            }
        }
        for item in &mut self.trash {
            if let VaultItem::DomainGroup(dg) = item {
                dg.accounts.retain(|account| {
                    if let Some(tombstone) = tombstones.iter().find(|t| t.id == account.id) {
                        account.updated_at > tombstone.deleted_at
                    } else {
                        true
                    }
                });
            }
        }
    }

    /// Exports the active vault entries as an RFC 4180-compliant CSV string.
    /// Excludes items currently in the trash.
    pub fn to_csv(&self) -> String {
        let mut csv = String::from("name,url,username,password,note\r\n");

        let escape_csv_value = |val: &str| -> String {
            if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
                let escaped = val.replace('"', "\"\"");
                format!("\"{}\"", escaped)
            } else {
                val.to_string()
            }
        };

        for item in &self.items {
            match item {
                VaultItem::DomainGroup(dg) => {
                    let name = escape_csv_value(&dg.title);
                    let url = escape_csv_value(dg.urls.first().map(|s| s.as_str()).unwrap_or(""));
                    for acc in &dg.accounts {
                        let username = escape_csv_value(&acc.username);
                        let password = escape_csv_value(&acc.password);
                        let note = escape_csv_value(&acc.notes);
                        csv.push_str(&format!(
                            "{},{},{},{},{}\r\n",
                            name, url, username, password, note
                        ));
                    }
                }
                VaultItem::SecureNote(sn) => {
                    let name = escape_csv_value(&sn.title);
                    let url = "";
                    let username = "";
                    let password = "";
                    let note = escape_csv_value(&sn.notes);
                    csv.push_str(&format!(
                        "{},{},{},{},{}\r\n",
                        name, url, username, password, note
                    ));
                }
            }
        }

        csv
    }

    /// Parses a CSV string and imports entries into the active entries.
    /// Returns the number of successfully imported entries, or an error.
    pub fn import_csv(&mut self, csv_text: &str, current_time: &str) -> Result<usize, String> {
        let rows = parse_csv_rows(csv_text)?;
        if rows.is_empty() {
            return Err("CSV is empty".to_string());
        }

        let headers = &rows[0];

        let mut name_idx = None;
        let mut url_idx = None;
        let mut username_idx = None;
        let mut password_idx = None;
        let mut note_idx = None;

        for (i, header) in headers.iter().enumerate() {
            let h = header.trim().to_lowercase();
            if h == "name" || h == "title" {
                name_idx = Some(i);
            } else if h == "url" || h == "urls" || h == "website" {
                url_idx = Some(i);
            } else if h == "username" || h == "user" || h == "email" {
                username_idx = Some(i);
            } else if h == "password" || h == "pass" {
                password_idx = Some(i);
            } else if h == "note" || h == "notes" {
                note_idx = Some(i);
            }
        }

        let name_idx = name_idx
            .ok_or_else(|| "Invalid CSV: 'name' or 'title' column is required".to_string())?;

        let mut import_count = 0;
        for row in rows.iter().skip(1) {
            if row.is_empty() {
                continue;
            }

            let title = row
                .get(name_idx)
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string();
            if title.is_empty() {
                continue;
            }

            let url = url_idx
                .and_then(|idx| row.get(idx))
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string();
            let username = username_idx
                .and_then(|idx| row.get(idx))
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string();
            let password = password_idx
                .and_then(|idx| row.get(idx))
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string();
            let notes = note_idx
                .and_then(|idx| row.get(idx))
                .cloned()
                .unwrap_or_default()
                .to_string();

            let urls = if url.is_empty() {
                Vec::new()
            } else {
                vec![url.clone()]
            };

            if username.is_empty() && password.is_empty() && urls.is_empty() {
                let note = SecureNote {
                    id: generate_uuid_v4(),
                    title,
                    notes,
                    tags: Vec::new(),
                    created_at: current_time.to_string(),
                    updated_at: current_time.to_string(),
                };
                self.items.push(VaultItem::SecureNote(note));
            } else {
                let base_domain = get_base_domain(&url).unwrap_or_else(|| title.clone());

                let mut found_dg_idx = None;
                for (idx, item) in self.items.iter().enumerate() {
                    if let VaultItem::DomainGroup(dg) = item
                        && dg.title.to_lowercase() == base_domain.to_lowercase() {
                            found_dg_idx = Some(idx);
                            break;
                        }
                }

                let new_account = Account {
                    id: generate_uuid_v4(),
                    username,
                    password,
                    totp_secret: None,
                    notes,
                    custom_fields: Vec::new(),
                    password_history: Vec::new(),
                    created_at: current_time.to_string(),
                    updated_at: current_time.to_string(),
                };

                if let Some(idx) = found_dg_idx {
                    if let VaultItem::DomainGroup(dg) = &mut self.items[idx] {
                        dg.accounts.push(new_account);
                        dg.updated_at = current_time.to_string();
                    }
                } else {
                    let new_dg = DomainGroup {
                        id: generate_uuid_v4(),
                        title: base_domain,
                        urls,
                        accounts: vec![new_account],
                        tags: Vec::new(),
                        created_at: current_time.to_string(),
                        updated_at: current_time.to_string(),
                    };
                    self.items.push(VaultItem::DomainGroup(new_dg));
                }
            }
            import_count += 1;
        }

        Ok(import_count)
    }
}

fn get_base_domain(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    let without_protocol = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("ftp://");

    let host = without_protocol.split('/').next()?.split(':').next()?;
    let host = host.trim_start_matches("www.");
    if host.is_empty() {
        return None;
    }
    Some(host.to_string())
}

fn parse_csv_rows(csv: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = csv.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current_field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(c);
            }
        } else {
            match c {
                '"' => {
                    in_quotes = true;
                }
                ',' => {
                    current_row.push(current_field.clone());
                    current_field.clear();
                }
                '\n' => {
                    current_row.push(current_field.clone());
                    current_field.clear();
                    rows.push(current_row.clone());
                    current_row.clear();
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    current_row.push(current_field.clone());
                    current_field.clear();
                    rows.push(current_row.clone());
                    current_row.clear();
                }
                _ => {
                    current_field.push(c);
                }
            }
        }
    }

    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
        rows.push(current_row);
    }

    if in_quotes {
        return Err("Malformed CSV: unclosed double quotes".to_string());
    }

    Ok(rows)
}

fn generate_uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    if crate::crypto::generate_random_bytes(&mut bytes).is_err() {
        use rand::Rng;
        rand::rng().fill_bytes(&mut bytes);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_item_serialization() {
        let entry = DomainGroup {
            id: "uuid-1234".to_string(),
            title: "My Email".to_string(),
            urls: vec!["https://gmail.com".to_string()],
            accounts: vec![Account {
                id: "acc-uuid".to_string(),
                username: "user@gmail.com".to_string(),
                password: "super-secure-password".to_string(),
                totp_secret: Some("TOTP123".to_string()),
                notes: "My primary email account.".to_string(),
                custom_fields: vec![CustomField {
                    key: "PinCode".to_string(),
                    value: "9988".to_string(),
                }],
                password_history: vec![],
                created_at: "2026-06-30T12:00:00Z".to_string(),
                updated_at: "2026-06-30T12:00:00Z".to_string(),
            }],
            tags: vec!["personal".to_string()],
            created_at: "2026-06-30T12:00:00Z".to_string(),
            updated_at: "2026-06-30T12:00:00Z".to_string(),
        };

        let item = VaultItem::DomainGroup(entry.clone());

        // Serialize to a JSON String
        let json = serde_json::to_string(&item).expect("Failed to serialize to JSON");

        // Assert that the JSON contains our data keys
        assert!(json.contains("\"type\":\"DomainGroup\""));
        assert!(json.contains("\"title\":\"My Email\""));
        assert!(json.contains("\"username\":\"user@gmail.com\""));

        // Deserialize back to a VaultItem struct
        let deserialized: VaultItem =
            serde_json::from_str(&json).expect("Failed to deserialize JSON");

        // Verify that the deserialized entry matches the original exactly!
        assert_eq!(item, deserialized);
    }

    #[test]
    fn test_vault_crud_operations() {
        let mut vault = Vault::new();
        let group_id = "group-1".to_string();

        let dg = DomainGroup {
            id: group_id.clone(),
            title: "Test Site".to_string(),
            urls: vec![],
            accounts: vec![Account {
                id: "acc-1".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
                totp_secret: None,
                notes: "".to_string(),
                custom_fields: vec![],
                password_history: vec![],
                created_at: "2026-06-30T12:00:00Z".to_string(),
                updated_at: "2026-06-30T12:00:00Z".to_string(),
            }],
            tags: vec![],
            created_at: "2026-06-30T12:00:00Z".to_string(),
            updated_at: "2026-06-30T12:00:00Z".to_string(),
        };

        // 1. Test Add
        vault.add_item(VaultItem::DomainGroup(dg));
        assert_eq!(vault.items.len(), 1);

        // 2. Test Update
        let mut updated = match &vault.items[0] {
            VaultItem::DomainGroup(dg) => dg.clone(),
            _ => panic!("wrong type"),
        };
        updated.accounts[0].password = "new-pass".to_string();
        vault.update_item(VaultItem::DomainGroup(updated)).unwrap();

        match &vault.items[0] {
            VaultItem::DomainGroup(dg) => {
                assert_eq!(dg.accounts[0].password, "new-pass");
            }
            _ => panic!("wrong type"),
        }

        // 3. Test Delete (moves to trash)
        vault
            .delete_item(&group_id, "2026-06-30T12:05:00Z")
            .unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 1);
        assert_eq!(vault.tombstones.len(), 0);

        // 4. Test Restore
        vault
            .restore_item(&group_id, "2026-06-30T12:06:00Z")
            .unwrap();
        assert_eq!(vault.items.len(), 1);
        assert_eq!(vault.trash.len(), 0);

        // 5. Test Purge
        vault.purge_item(&group_id, "2026-06-30T12:07:00Z").unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 0);
        assert_eq!(vault.tombstones.len(), 1);
        assert_eq!(vault.tombstones[0].id, group_id);
    }

    #[test]
    fn test_vault_idempotent_operations() {
        let mut vault = Vault::new();
        let group_id = "test-group".to_string();
        let dg = DomainGroup {
            id: group_id.clone(),
            title: "Test Group".to_string(),
            urls: vec![],
            accounts: vec![],
            tags: vec![],
            created_at: "2026-06-30T12:00:00Z".to_string(),
            updated_at: "2026-06-30T12:00:00Z".to_string(),
        };

        // Add
        vault.add_item(VaultItem::DomainGroup(dg));

        // Delete once
        vault
            .delete_item(&group_id, "2026-06-30T12:05:00Z")
            .unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 1);

        // Delete second time (should be idempotent Ok(()))
        vault
            .delete_item(&group_id, "2026-06-30T12:05:00Z")
            .unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 1);

        // Restore once
        vault
            .restore_item(&group_id, "2026-06-30T12:06:00Z")
            .unwrap();
        assert_eq!(vault.items.len(), 1);
        assert_eq!(vault.trash.len(), 0);

        // Restore second time (should be idempotent Ok(()))
        vault
            .restore_item(&group_id, "2026-06-30T12:06:00Z")
            .unwrap();
        assert_eq!(vault.items.len(), 1);
        assert_eq!(vault.trash.len(), 0);

        // Purge once
        vault.purge_item(&group_id, "2026-06-30T12:07:00Z").unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 0);
        assert_eq!(vault.tombstones.len(), 1);

        // Purge second time (should be idempotent Ok(()) and not duplicate tombstones)
        vault.purge_item(&group_id, "2026-06-30T12:07:00Z").unwrap();
        assert_eq!(vault.items.len(), 0);
        assert_eq!(vault.trash.len(), 0);
        assert_eq!(vault.tombstones.len(), 1);
    }

    #[test]
    fn test_vault_merge() {
        let mut vault_a = Vault::new();
        let mut vault_b = Vault::new();

        let group_id = "shared-group".to_string();

        // Vault A has an entry updated at 12:00
        vault_a.add_item(VaultItem::DomainGroup(DomainGroup {
            id: group_id.clone(),
            title: "Vault A Version".to_string(),
            urls: vec![],
            accounts: vec![Account {
                id: "acc-1".to_string(),
                username: "user".to_string(),
                password: "pass-a".to_string(),
                totp_secret: None,
                notes: "".to_string(),
                custom_fields: vec![],
                password_history: vec![],
                created_at: "2026-06-30T12:00:00Z".to_string(),
                updated_at: "2026-06-30T12:00:00Z".to_string(),
            }],
            tags: vec![],
            created_at: "2026-06-30T12:00:00Z".to_string(),
            updated_at: "2026-06-30T12:00:00Z".to_string(),
        }));

        // Vault B has an entry updated at 12:05 (newer)
        vault_b.add_item(VaultItem::DomainGroup(DomainGroup {
            id: group_id.clone(),
            title: "Vault B Version".to_string(),
            urls: vec![],
            accounts: vec![Account {
                id: "acc-1".to_string(),
                username: "user".to_string(),
                password: "pass-b".to_string(),
                totp_secret: None,
                notes: "".to_string(),
                custom_fields: vec![],
                password_history: vec![],
                created_at: "2026-06-30T12:00:00Z".to_string(),
                updated_at: "2026-06-30T12:05:00Z".to_string(),
            }],
            tags: vec![],
            created_at: "2026-06-30T12:00:00Z".to_string(),
            updated_at: "2026-06-30T12:05:00Z".to_string(),
        }));

        // Merge B into A. B's newer edit must overwrite A's edit.
        vault_a.merge(vault_b);
        assert_eq!(vault_a.items.len(), 1);
        match &vault_a.items[0] {
            VaultItem::DomainGroup(dg) => {
                assert_eq!(dg.accounts[0].password, "pass-b");
            }
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn test_vault_to_csv() {
        let mut vault = Vault::new();

        vault.add_item(VaultItem::DomainGroup(DomainGroup {
            id: "uuid-1".to_string(),
            title: "Github".to_string(),
            urls: vec!["https://github.com".to_string()],
            accounts: vec![Account {
                id: "acc-1".to_string(),
                username: "octocat".to_string(),
                password: "password123".to_string(),
                totp_secret: None,
                notes: "Github notes".to_string(),
                custom_fields: Vec::new(),
                password_history: Vec::new(),
                created_at: "2026-07-05T12:00:00Z".to_string(),
                updated_at: "2026-07-05T12:00:00Z".to_string(),
            }],
            tags: Vec::new(),
            created_at: "2026-07-05T12:00:00Z".to_string(),
            updated_at: "2026-07-05T12:00:00Z".to_string(),
        }));

        vault.add_item(VaultItem::SecureNote(SecureNote {
            id: "uuid-2".to_string(),
            title: "My, Note \"Special\"".to_string(),
            notes: "First line\nSecond line".to_string(),
            tags: Vec::new(),
            created_at: "2026-07-05T12:05:00Z".to_string(),
            updated_at: "2026-07-05T12:05:00Z".to_string(),
        }));

        let csv = vault.to_csv();
        let expected = "name,url,username,password,note\r\n\
                        Github,https://github.com,octocat,password123,Github notes\r\n\
                        \"My, Note \"\"Special\"\"\",,,,\"First line\nSecond line\"\r\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_vault_import_csv() {
        let mut vault = Vault::new();

        let csv_text = "Username,Password,NAME,notes,url\r\n\
                        octocat,password123,Github,Github notes,https://github.com\r\n\
                        ,,\"My, Note \"\"Special\"\"\",\"First line\nSecond line\",\r\n";

        let count = vault.import_csv(csv_text, "2026-07-05T19:00:00Z").unwrap();
        assert_eq!(count, 2);
        assert_eq!(vault.items.len(), 2);

        match &vault.items[0] {
            VaultItem::DomainGroup(dg) => {
                assert_eq!(dg.title, "github.com");
                assert_eq!(dg.accounts[0].username, "octocat");
                assert_eq!(dg.accounts[0].password, "password123");
            }
            _ => panic!("wrong type"),
        }

        match &vault.items[1] {
            VaultItem::SecureNote(sn) => {
                assert_eq!(sn.title, "My, Note \"Special\"");
                assert_eq!(sn.notes, "First line\nSecond line");
            }
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn test_vault_merge_trash_conflict() {
        // Case 1: Trash version is newer -> should go to trash
        let mut vault_a = Vault::new();
        let mut vault_b = Vault::new();
        let item_id = "test-item-1".to_string();

        let dg_active = DomainGroup {
            id: item_id.clone(),
            title: "Active Site".to_string(),
            urls: vec![],
            accounts: vec![],
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-01T12:00:00Z".to_string(),
        };
        vault_a.add_item(VaultItem::DomainGroup(dg_active));

        let dg_trash = DomainGroup {
            id: item_id.clone(),
            title: "Active Site".to_string(),
            urls: vec![],
            accounts: vec![],
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-02T12:00:00Z".to_string(), // Newer
        };
        vault_b.trash.push(VaultItem::DomainGroup(dg_trash));

        vault_a.merge(vault_b);
        assert_eq!(vault_a.items.len(), 0);
        assert_eq!(vault_a.trash.len(), 1);
        assert_eq!(vault_a.trash[0].id(), item_id);

        // Case 2: Active version is newer -> should be active and removed from trash
        let mut vault_c = Vault::new();
        let mut vault_d = Vault::new();

        let dg_trash_older = DomainGroup {
            id: item_id.clone(),
            title: "Trash Site".to_string(),
            urls: vec![],
            accounts: vec![],
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-01T12:00:00Z".to_string(),
        };
        vault_c.trash.push(VaultItem::DomainGroup(dg_trash_older));

        let dg_active_newer = DomainGroup {
            id: item_id.clone(),
            title: "Trash Site".to_string(),
            urls: vec![],
            accounts: vec![],
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-02T12:00:00Z".to_string(), // Newer
        };
        vault_d.add_item(VaultItem::DomainGroup(dg_active_newer));

        vault_c.merge(vault_d);
        assert_eq!(vault_c.items.len(), 1);
        assert_eq!(vault_c.trash.len(), 0);
        assert_eq!(vault_c.items[0].id(), item_id);
    }

    #[test]
    fn test_vault_merge_tombstones() {
        // Case 1: Active item with tombstone. Tombstone is newer -> item removed
        let mut vault_a = Vault::new();
        let mut vault_b = Vault::new();
        let item_id = "test-item-2".to_string();

        let sn = SecureNote {
            id: item_id.clone(),
            title: "My Note".to_string(),
            notes: "Some note content".to_string(),
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-01T12:00:00Z".to_string(),
        };
        vault_a.add_item(VaultItem::SecureNote(sn));

        vault_b.tombstones.push(Tombstone {
            id: item_id.clone(),
            deleted_at: "2026-07-02T12:00:00Z".to_string(), // Newer
        });

        vault_a.merge(vault_b);
        assert_eq!(vault_a.items.len(), 0);
        assert_eq!(vault_a.tombstones.len(), 1);

        // Case 2: Active item with tombstone. Item updated_at is newer -> item kept
        let mut vault_c = Vault::new();
        let mut vault_d = Vault::new();

        let sn_newer = SecureNote {
            id: item_id.clone(),
            title: "My Note".to_string(),
            notes: "Updated note content".to_string(),
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-03T12:00:00Z".to_string(), // Newer than tombstone
        };
        vault_c.add_item(VaultItem::SecureNote(sn_newer));

        vault_d.tombstones.push(Tombstone {
            id: item_id.clone(),
            deleted_at: "2026-07-02T12:00:00Z".to_string(), // Older
        });

        vault_c.merge(vault_d);
        assert_eq!(vault_c.items.len(), 1);
        assert_eq!(vault_c.items[0].id(), item_id);

        // Case 3: Account tombstone. Account in DomainGroup. Tombstone is newer -> account removed
        let mut vault_e = Vault::new();
        let mut vault_f = Vault::new();
        let acc_id = "account-1".to_string();

        let dg = DomainGroup {
            id: "group-1".to_string(),
            title: "My Domain".to_string(),
            urls: vec![],
            accounts: vec![Account {
                id: acc_id.clone(),
                username: "user".to_string(),
                password: "pass".to_string(),
                totp_secret: None,
                notes: "".to_string(),
                custom_fields: vec![],
                password_history: vec![],
                created_at: "2026-07-01T12:00:00Z".to_string(),
                updated_at: "2026-07-01T12:00:00Z".to_string(),
            }],
            tags: vec![],
            created_at: "2026-07-01T12:00:00Z".to_string(),
            updated_at: "2026-07-01T12:00:00Z".to_string(),
        };
        vault_e.add_item(VaultItem::DomainGroup(dg));

        vault_f.tombstones.push(Tombstone {
            id: acc_id.clone(),
            deleted_at: "2026-07-02T12:00:00Z".to_string(), // Newer
        });

        vault_e.merge(vault_f);
        assert_eq!(vault_e.items.len(), 1);
        if let VaultItem::DomainGroup(merged_dg) = &vault_e.items[0] {
            assert_eq!(merged_dg.accounts.len(), 0);
        } else {
            panic!("Expected DomainGroup");
        }
    }

    #[test]
    fn test_vault_merge_identical_timestamps() {
        let mut vault_a = Vault::new();
        let mut vault_b = Vault::new();
        let group_id = "shared-group".to_string();
        let timestamp = "2026-07-01T12:00:00Z".to_string();

        vault_a.add_item(VaultItem::DomainGroup(DomainGroup {
            id: group_id.clone(),
            title: "Domain Group".to_string(),
            urls: vec![],
            accounts: vec![Account {
                id: "acc-a".to_string(),
                username: "userA".to_string(),
                password: "passA".to_string(),
                totp_secret: None,
                notes: "".to_string(),
                custom_fields: vec![],
                password_history: vec![],
                created_at: timestamp.clone(),
                updated_at: timestamp.clone(),
            }],
            tags: vec![],
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
        }));

        vault_b.add_item(VaultItem::DomainGroup(DomainGroup {
            id: group_id.clone(),
            title: "Domain Group".to_string(),
            urls: vec![],
            accounts: vec![Account {
                id: "acc-b".to_string(),
                username: "userB".to_string(),
                password: "passB".to_string(),
                totp_secret: None,
                notes: "".to_string(),
                custom_fields: vec![],
                password_history: vec![],
                created_at: timestamp.clone(),
                updated_at: timestamp.clone(),
            }],
            tags: vec![],
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
        }));

        vault_a.merge(vault_b);
        assert_eq!(vault_a.items.len(), 1);
        if let VaultItem::DomainGroup(merged_dg) = &vault_a.items[0] {
            assert_eq!(merged_dg.accounts.len(), 2);
            let ids: Vec<&str> = merged_dg.accounts.iter().map(|a| a.id.as_str()).collect();
            assert!(ids.contains(&"acc-a"));
            assert!(ids.contains(&"acc-b"));
        } else {
            panic!("Expected DomainGroup");
        }
    }

    #[test]
    fn test_vault_import_csv_malformed() {
        let mut vault = Vault::new();
        // Unclosed double quotes at the end of row
        let malformed_csv =
            "name,url,username,password,note\n\"Github,https://github.com,octocat,pass,note";
        let result = vault.import_csv(malformed_csv, "2026-07-05T19:00:00Z");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Malformed CSV: unclosed double quotes");
    }

    #[test]
    fn test_vault_import_csv_missing_columns() {
        let mut vault = Vault::new();
        // Missing url and note columns completely
        let csv_text = "name,username,password\nGithub,octocat,password123";
        let count = vault.import_csv(csv_text, "2026-07-05T19:00:00Z").unwrap();
        assert_eq!(count, 1);
        assert_eq!(vault.items.len(), 1);

        if let VaultItem::DomainGroup(dg) = &vault.items[0] {
            assert_eq!(dg.title, "Github");
            assert_eq!(dg.urls.len(), 0);
            assert_eq!(dg.accounts[0].username, "octocat");
            assert_eq!(dg.accounts[0].password, "password123");
            assert_eq!(dg.accounts[0].notes, "");
        } else {
            panic!("Expected DomainGroup");
        }
    }
}
