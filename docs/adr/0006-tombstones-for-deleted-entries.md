# 0006. Tombstones for Deleted Entries

## Status
Accepted

## Context
When performing client-side sync merging, the merge engine needs to distinguish between an entry that was deleted on one device and an entry that was newly created on another. Without explicit deletion tracking, deleted entries would be treated as missing on one side and reinstated by the merge engine, causing them to "resurrect."

## Decision
We will include a `tombstones` list to track deleted items, extending this strategy to both top-level items and nested accounts:
1. **Definition:** A tombstone is a record containing the `id` (UUID v4) and `deleted_at` (DateTime) of a deleted item or nested account.
2. **Deletions:** When a user deletes a top-level item (e.g. a Domain Group or Secure Note), its ID and deletion timestamp are appended to the vault-level `tombstones` list. When a user deletes a specific account within a Domain Group, the account ID and deletion timestamp are appended to the group-level `tombstones` list.
3. **Merging:** During sync merging, an item or nested account is discarded if its ID matches a tombstone whose `deleted_at` timestamp is newer than the item/account's `updated_at` timestamp.
4. **Pruning:** Tombstones older than 30 days will be automatically purged from the metadata lists to prevent infinite growth, assuming active client devices sync at least once every 30 days.

## Consequences
- Guarantees that deletions sync reliably across all devices.
- Prevents deleted entries from resurrecting during merges.
- Minimal storage overhead which is periodically pruned.
