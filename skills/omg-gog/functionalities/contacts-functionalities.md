# Functionalities: Contacts

## Overview
Google Contacts via People API — contact CRUD, search, directory listing/search, and "other contacts" listing/search.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `contacts search <query>` | `handle_contacts_search` | src/cli/mod.rs:8052 | Search contacts |
| 2 | `contacts list` | `handle_contacts_list` | src/cli/mod.rs:8079 | List all contacts |
| 3 | `contacts get <resource_name>` | `handle_contacts_get` | src/cli/mod.rs:8106 | Get contact details |
| 4 | `contacts create` | `handle_contacts_create` | src/cli/mod.rs:8129 | Create contact (name, email, phone) |
| 5 | `contacts update <resource_name>` | `handle_contacts_update` | src/cli/mod.rs:8163 | Update contact |
| 6 | `contacts delete <resource_name>` | `handle_contacts_delete` | src/cli/mod.rs:8205 | Delete contact |
| 7 | `contacts directory list` | `handle_contacts_directory_list` | src/cli/mod.rs:8238 | List directory contacts |
| 8 | `contacts directory search` | `handle_contacts_directory_search` | src/cli/mod.rs:8265 | Search directory |
| 9 | `contacts other list` | `handle_contacts_other_list` | src/cli/mod.rs:8292 | List other contacts |
| 10 | `contacts other search` | `handle_contacts_other_search` | src/cli/mod.rs:8321 | Search other contacts |

## URL/Body Builders

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `build_contacts_list_url` | src/services/contacts/contacts.rs | Contacts list URL |
| 2 | `build_contact_get_url` | src/services/contacts/contacts.rs | Contact get URL |
| 3 | `build_contact_create_url` | src/services/contacts/contacts.rs | Contact create URL |
| 4 | `build_contact_create_body` | src/services/contacts/contacts.rs | Contact create body |
| 5 | `build_contact_update_url` | src/services/contacts/contacts.rs | Contact update URL |
| 6 | `build_contact_delete_url` | src/services/contacts/contacts.rs | Contact delete URL |
| 7 | `build_contacts_search_url` | src/services/contacts/contacts.rs | Contact search URL |
| 8 | `build_directory_list_url` | src/services/contacts/directory.rs | Directory list URL |
| 9 | `build_directory_search_url` | src/services/contacts/directory.rs | Directory search URL |
| 10 | `build_other_contacts_list_url` | src/services/contacts/directory.rs | Other contacts list URL |
| 11 | `build_other_contacts_search_url` | src/services/contacts/directory.rs | Other contacts search URL |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Person | Struct | src/services/contacts/types.rs | Contact person |
| 2 | PersonName | Struct | src/services/contacts/types.rs | Name fields |
| 3 | EmailAddress | Struct | src/services/contacts/types.rs | Email address |
| 4 | PhoneNumber | Struct | src/services/contacts/types.rs | Phone number |
| 5 | Birthday | Struct | src/services/contacts/types.rs | Birthday |
| 6 | DateValue | Struct | src/services/contacts/types.rs | Date components |
| 7 | Biography | Struct | src/services/contacts/types.rs | Bio/notes |
| 8 | Photo | Struct | src/services/contacts/types.rs | Profile photo |
| 9 | PersonListResponse | Struct | src/services/contacts/types.rs | Contact list |
| 10 | DirectoryListResponse | Struct | src/services/contacts/types.rs | Directory list |
