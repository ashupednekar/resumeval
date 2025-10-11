# Requirements Document

## Introduction

This feature addresses critical issues in the resume evaluation system related to file upload functionality, API integration, and job management. The system currently has non-functional drag & drop resume upload, hardcoded task data instead of API calls, and broken job update functionality. This spec will establish proper file upload handling, API integration for dynamic task management, and fix the job update workflow.

## Requirements

### Requirement 1

**User Story:** As a recruiter, I want to upload resume files via drag & drop interface, so that I can efficiently process multiple candidate resumes for evaluation.

#### Acceptance Criteria

1. WHEN a user drags files over the upload area THEN the system SHALL highlight the drop zone with visual feedback
2. WHEN a user drops PDF or DOC files in the upload area THEN the system SHALL accept and process the files
3. WHEN invalid file types are dropped THEN the system SHALL display an error message and reject the files
4. WHEN files are successfully uploaded THEN the system SHALL display upload progress and confirmation
5. WHEN the upload completes THEN the system SHALL create evaluation records and update the task status

### Requirement 2

**User Story:** As a recruiter, I want the system to fetch evaluation tasks from the backend API, so that I can see real-time task status and data instead of hardcoded placeholders.

#### Acceptance Criteria

1. WHEN the dashboard loads THEN the system SHALL fetch evaluation tasks from the `/evaluations` API endpoint
2. WHEN task data is received THEN the system SHALL display actual task information including status, progress, and metrics
3. WHEN the API call fails THEN the system SHALL display an appropriate error message and retry mechanism
4. WHEN tasks are updated THEN the system SHALL refresh the task list automatically
5. IF no tasks exist THEN the system SHALL display an empty state with guidance to create new evaluations

### Requirement 3

**User Story:** As a recruiter, I want to update existing job descriptions, so that I can modify job requirements and details as needed.

#### Acceptance Criteria

1. WHEN a user clicks on a job card THEN the system SHALL display job details with edit capabilities
2. WHEN a user modifies job fields THEN the system SHALL validate the input data
3. WHEN a user saves job changes THEN the system SHALL send a PATCH request to update the job
4. WHEN the job update succeeds THEN the system SHALL refresh the job list and show success notification
5. WHEN the job update fails THEN the system SHALL display error messages and maintain the edit state

### Requirement 4

**User Story:** As a system administrator, I want proper file handling and storage, so that uploaded resumes are securely stored and accessible for processing.

#### Acceptance Criteria

1. WHEN files are uploaded THEN the system SHALL validate file size limits (max 10MB per file)
2. WHEN files are processed THEN the system SHALL store them in a secure location with proper naming
3. WHEN files are stored THEN the system SHALL create database records linking files to evaluation tasks
4. WHEN file processing fails THEN the system SHALL clean up partial uploads and notify the user
5. WHEN files are accessed THEN the system SHALL verify user permissions and project access

### Requirement 5

**User Story:** As a recruiter, I want real-time feedback during file operations, so that I understand the system status and can take appropriate actions.

#### Acceptance Criteria

1. WHEN file upload starts THEN the system SHALL display a progress indicator
2. WHEN upload progress changes THEN the system SHALL update the progress bar in real-time
3. WHEN operations complete successfully THEN the system SHALL show success notifications
4. WHEN errors occur THEN the system SHALL display specific error messages with suggested actions
5. WHEN multiple files are processed THEN the system SHALL show individual file status and overall progress