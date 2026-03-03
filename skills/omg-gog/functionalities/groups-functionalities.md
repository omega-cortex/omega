# Functionalities: Groups

## Overview
Google Cloud Identity Groups API — group listing and membership listing.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `groups list` | `handle_groups_list` | src/cli/mod.rs:8511 | List groups |
| 2 | `groups members <group_email>` | `handle_groups_members` | src/cli/mod.rs:8568 | List group members |

## URL Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_groups_list_url` | src/services/groups/groups.rs | Groups list URL |
| 2 | `build_group_lookup_url` | src/services/groups/groups.rs | Group lookup by email |
| 3 | `build_memberships_list_url` | src/services/groups/groups.rs | Membership list URL |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Group | Struct | src/services/groups/types.rs | Group record |
| 2 | GroupKey | Struct | src/services/groups/types.rs | Group key (email) |
| 3 | GroupListResponse | Struct | src/services/groups/types.rs | Group list |
| 4 | Membership | Struct | src/services/groups/types.rs | Membership record |
| 5 | MemberKey | Struct | src/services/groups/types.rs | Member key (email) |
| 6 | MembershipRole | Struct | src/services/groups/types.rs | Role in group |
| 7 | MembershipListResponse | Struct | src/services/groups/types.rs | Membership list |
