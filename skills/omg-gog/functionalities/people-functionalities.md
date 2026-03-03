# Functionalities: People

## Overview
Google People API — profile retrieval (self/other), search, and relationship/connection listing.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `people me` | `handle_people_me` | src/cli/mod.rs:8369 | Get own profile |
| 2 | `people get <resource_name>` | `handle_people_get` | src/cli/mod.rs:8395 | Get person by resource name |
| 3 | `people search <query>` | `handle_people_search` | src/cli/mod.rs:8424 | Search people |
| 4 | `people relations` | `handle_people_relations` | src/cli/mod.rs:8460 | List connections/relations |

## URL Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_people_me_url` | src/services/people/people.rs | Self profile URL |
| 2 | `build_people_get_url` | src/services/people/people.rs | Person get URL |
| 3 | `build_people_search_url` | src/services/people/people.rs | People search URL |
| 4 | `build_people_connections_url` | src/services/people/people.rs | Connections list URL |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | PersonResponse | Struct | src/services/people/types.rs | Person response |
| 2 | PersonName | Struct | src/services/people/types.rs | Name fields |
| 3 | EmailAddress | Struct | src/services/people/types.rs | Email |
| 4 | Photo | Struct | src/services/people/types.rs | Profile photo |
| 5 | Locale | Struct | src/services/people/types.rs | Locale info |
| 6 | SearchResponse | Struct | src/services/people/types.rs | Search results |
| 7 | SearchResult | Struct | src/services/people/types.rs | Individual result |
| 8 | Relation | Struct | src/services/people/types.rs | Relationship |
