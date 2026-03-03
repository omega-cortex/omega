# Functionalities: Classroom

## Overview
Google Classroom API — the largest service module. 13 command groups covering courses, roster (students/teachers), coursework, course materials, submissions, announcements, topics, invitations, guardians, and user profiles. ~60+ leaf commands.

## CLI Command Groups

### Courses (10 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom courses list` | `handle_classroom_courses_list` | src/cli/mod.rs:5920 | List courses |
| 2 | `classroom courses get <id>` | `handle_classroom_courses_get` | src/cli/mod.rs:5947 | Get course details |
| 3 | `classroom courses create` | `handle_classroom_courses_create` | src/cli/mod.rs:5975 | Create course |
| 4 | `classroom courses update <id>` | `handle_classroom_courses_update` | src/cli/mod.rs:6016 | Update course |
| 5 | `classroom courses patch <id>` | inline | src/cli/mod.rs | Patch course fields |
| 6 | `classroom courses delete <id>` | `handle_classroom_courses_delete` | src/cli/mod.rs:6079 | Delete course |
| 7 | `classroom courses archive <id>` | inline | src/cli/mod.rs | Archive course |
| 8 | `classroom courses activate <id>` | inline | src/cli/mod.rs | Activate course |
| 9 | `classroom courses aliases list <id>` | inline | src/cli/mod.rs | List course aliases |
| 10 | `classroom courses aliases create <id>` | inline | src/cli/mod.rs | Create course alias |

### Students (4 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom students list <course_id>` | `handle_classroom_students_list` | src/cli/mod.rs:6114 | List students |
| 2 | `classroom students get <course_id> <user_id>` | `handle_classroom_students_get` | src/cli/mod.rs:6141 | Get student |
| 3 | `classroom students add <course_id> <email>` | inline | src/cli/mod.rs | Enroll student |
| 4 | `classroom students remove <course_id> <user_id>` | inline | src/cli/mod.rs | Remove student |

### Teachers (4 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom teachers list <course_id>` | `handle_classroom_teachers_list` | src/cli/mod.rs:6219 | List teachers |
| 2 | `classroom teachers get <course_id> <user_id>` | `handle_classroom_teachers_get` | src/cli/mod.rs:6246 | Get teacher |
| 3 | `classroom teachers add <course_id> <email>` | inline | src/cli/mod.rs | Add teacher |
| 4 | `classroom teachers remove <course_id> <user_id>` | inline | src/cli/mod.rs | Remove teacher |

### Roster (1 command)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom roster <course_id>` | inline | src/cli/mod.rs | List all students + teachers |

### Coursework (6 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom coursework list <course_id>` | inline | src/cli/mod.rs | List coursework |
| 2 | `classroom coursework get <course_id> <id>` | inline | src/cli/mod.rs | Get coursework |
| 3 | `classroom coursework create <course_id>` | inline | src/cli/mod.rs | Create coursework |
| 4 | `classroom coursework update <course_id> <id>` | inline | src/cli/mod.rs | Update coursework |
| 5 | `classroom coursework delete <course_id> <id>` | inline | src/cli/mod.rs | Delete coursework |
| 6 | `classroom coursework patch <course_id> <id>` | inline | src/cli/mod.rs | Patch coursework |

### Materials (5 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom materials list <course_id>` | inline | src/cli/mod.rs | List course materials |
| 2 | `classroom materials get <course_id> <id>` | inline | src/cli/mod.rs | Get material |
| 3 | `classroom materials create <course_id>` | inline | src/cli/mod.rs | Create material |
| 4 | `classroom materials update/patch <course_id> <id>` | inline | src/cli/mod.rs | Update material |
| 5 | `classroom materials delete <course_id> <id>` | inline | src/cli/mod.rs | Delete material |

### Submissions (6 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom submissions list <course_id> <cw_id>` | inline | src/cli/mod.rs | List submissions |
| 2 | `classroom submissions get <course_id> <cw_id> <id>` | inline | src/cli/mod.rs | Get submission |
| 3 | `classroom submissions grade <...>` | inline | src/cli/mod.rs | Assign grade |
| 4 | `classroom submissions return <...>` | inline | src/cli/mod.rs | Return submission |
| 5 | `classroom submissions reclaim <...>` | inline | src/cli/mod.rs | Reclaim submission |
| 6 | `classroom submissions turn-in <...>` | inline | src/cli/mod.rs | Turn in submission |

### Announcements (6 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom announcements list <course_id>` | inline | src/cli/mod.rs | List announcements |
| 2 | `classroom announcements get <course_id> <id>` | inline | src/cli/mod.rs | Get announcement |
| 3 | `classroom announcements create <course_id>` | inline | src/cli/mod.rs | Create announcement |
| 4 | `classroom announcements update <course_id> <id>` | inline | src/cli/mod.rs | Update announcement |
| 5 | `classroom announcements delete <course_id> <id>` | inline | src/cli/mod.rs | Delete announcement |
| 6 | `classroom announcements patch <course_id> <id>` | inline | src/cli/mod.rs | Patch announcement |

### Topics (5 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom topics list <course_id>` | inline | src/cli/mod.rs | List topics |
| 2 | `classroom topics get <course_id> <id>` | inline | src/cli/mod.rs | Get topic |
| 3 | `classroom topics create <course_id>` | inline | src/cli/mod.rs | Create topic |
| 4 | `classroom topics update <course_id> <id>` | inline | src/cli/mod.rs | Update topic |
| 5 | `classroom topics delete <course_id> <id>` | inline | src/cli/mod.rs | Delete topic |

### Invitations (5 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom invitations list` | inline | src/cli/mod.rs | List invitations |
| 2 | `classroom invitations get <id>` | inline | src/cli/mod.rs | Get invitation |
| 3 | `classroom invitations create` | inline | src/cli/mod.rs | Create invitation |
| 4 | `classroom invitations accept <id>` | inline | src/cli/mod.rs | Accept invitation |
| 5 | `classroom invitations delete <id>` | inline | src/cli/mod.rs | Delete invitation |

### Guardians (3 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom guardians list <student_id>` | inline | src/cli/mod.rs | List guardians |
| 2 | `classroom guardians get <student_id> <id>` | inline | src/cli/mod.rs | Get guardian |
| 3 | `classroom guardians delete <student_id> <id>` | inline | src/cli/mod.rs | Remove guardian |

### Guardian Invitations (3 commands)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom guardian-invitations list <student_id>` | inline | src/cli/mod.rs | List guardian invitations |
| 2 | `classroom guardian-invitations create <student_id>` | inline | src/cli/mod.rs | Invite guardian |
| 3 | `classroom guardian-invitations patch <id>` | inline | src/cli/mod.rs | Modify invitation |

### Profile (1 command)

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `classroom profile <user_id>` | inline | src/cli/mod.rs | Get user profile |

## Service Modules

| Module | File | Description |
|--------|------|-------------|
| courses | src/services/classroom/courses.rs | Course CRUD, archive/activate, aliases |
| roster | src/services/classroom/roster.rs | Student/teacher listing, enrollment |
| coursework | src/services/classroom/coursework.rs | Coursework CRUD |
| materials | src/services/classroom/materials.rs | Course material CRUD |
| submissions | src/services/classroom/submissions.rs | Submission operations (grade, return, turn-in) |
| announcements | src/services/classroom/announcements.rs | Announcement CRUD |
| topics | src/services/classroom/topics.rs | Topic CRUD |
| invitations | src/services/classroom/invitations.rs | Course invitations |
| guardians | src/services/classroom/guardians.rs | Guardian management |
| types | src/services/classroom/types.rs | All serde data types |

## Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Course | Struct | src/services/classroom/types.rs | Course metadata |
| 2 | CourseListResponse | Struct | src/services/classroom/types.rs | Course list |
| 3 | UserProfile | Struct | src/services/classroom/types.rs | User profile |
| 4 | Name | Struct | src/services/classroom/types.rs | User name |
| 5 | Student | Struct | src/services/classroom/types.rs | Student record |
| 6 | StudentListResponse | Struct | src/services/classroom/types.rs | Student list |
| 7 | Teacher | Struct | src/services/classroom/types.rs | Teacher record |
| 8 | TeacherListResponse | Struct | src/services/classroom/types.rs | Teacher list |
| 9 | CourseWork | Struct | src/services/classroom/types.rs | Assignment/quiz |
| 10 | CourseWorkListResponse | Struct | src/services/classroom/types.rs | Coursework list |
| 11 | CourseMaterial | Struct | src/services/classroom/types.rs | Course material |
| 12 | CourseMaterialListResponse | Struct | src/services/classroom/types.rs | Material list |
| 13 | StudentSubmission | Struct | src/services/classroom/types.rs | Student submission |
| 14 | SubmissionListResponse | Struct | src/services/classroom/types.rs | Submission list |
| 15 | Announcement | Struct | src/services/classroom/types.rs | Course announcement |
| 16 | AnnouncementListResponse | Struct | src/services/classroom/types.rs | Announcement list |
| 17 | Topic | Struct | src/services/classroom/types.rs | Course topic |
| 18 | TopicListResponse | Struct | src/services/classroom/types.rs | Topic list |
| 19 | Invitation | Struct | src/services/classroom/types.rs | Course invitation |
| 20 | InvitationListResponse | Struct | src/services/classroom/types.rs | Invitation list |
| 21 | Guardian | Struct | src/services/classroom/types.rs | Guardian record |
| 22 | GuardianListResponse | Struct | src/services/classroom/types.rs | Guardian list |
| 23 | GuardianInvitation | Struct | src/services/classroom/types.rs | Guardian invitation |
| 24 | GuardianInvitationListResponse | Struct | src/services/classroom/types.rs | Guardian invitation list |
