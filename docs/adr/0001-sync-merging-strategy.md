# 0001. Sync Merging Strategy

## Status
Accepted

## Context
When syncing the vault via Google Drive, multiple devices can mutate their local vaults offline or concurrently, causing optimistic concurrency (ETag) conflicts. 

## Decision
Instead of forcing the user to make a binary choice between overwriting their local vault or the remote vault, the core will decrypt both vaults, perform a client-side merge of individual entries using Last-Write-Wins (LWW) based on entry UUIDs and update timestamps, and push the merged result back.

## Consequences
- Prevents data loss when changes are made concurrently on different devices.
- Requires keeping track of deleted entries (e.g. via tombstones) to prevent deleted entries from being resurrected during a merge.
