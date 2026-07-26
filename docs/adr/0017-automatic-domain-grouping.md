# Automatic Domain Grouping and Matching Rules

We decided to match and group new credential accounts under domain groups by comparing the base domain of the new URL (or the entry title if empty) against either the existing Domain Group's title or any of its associated URLs. Trashed items are excluded from matches to prevent active credentials from being saved directly to deleted entries, and new tags are merged to retain discoverability. This robust matching logic ensures that domain groups remain consolidated and properly grouped even if users rename the parent Domain Group title.
