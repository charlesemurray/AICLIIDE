# Session UUID Issue Analysis

## Problem
When creating a new session, we get `InternalServerException` from AWS backend.

## Root Cause
Each new session generates a fresh UUID (`uuid::Uuid::new_v4().to_string()`) but the AWS backend may not be properly handling new conversation initialization.

## Evidence
1. **UUID Generation**: Line 395 in `mod.rs` creates new UUID for each session
2. **Proper Passing**: UUID correctly flows through ConversationState → FigConversationState → API
3. **Backend Error**: AWS returns InternalServerException, not client-side error

## Solution
The issue is likely AWS backend session state management, not our UUID handling.

## Quick Fix
Try adding a small delay or retry mechanism for new sessions to allow backend initialization.
