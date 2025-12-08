# Frontend Architecture Documentation

## Overview

Better USTC II is a cross-platform mobile application built with Tauri v2, Vue 3, and TypeScript. It aims to provide a better user experience for USTC students to access the "Second Class" (extra-curricular activities) system.

## Tech Stack

- **Framework**: Vue 3 (Composition API)
- **Build Tool**: Vite
- **UI Library**: Vant 4 (Mobile-first UI components)
- **State Management**: Pinia
- **Routing**: Vue Router
- **CSS Utility**: UnoCSS
- **Cross-Platform**: Tauri v2 (Rust backend)

## Project Structure

```
src/
├── assets/          # Static assets (images, styles)
├── components/      # Reusable Vue components
│   └── ActivityCard.vue  # Card component for displaying activity info
├── router/          # Vue Router configuration
├── stores/          # Pinia stores
│   ├── activity.ts  # Activity data and logic
│   ├── logs.ts      # Application logging
│   └── user.ts      # User authentication and profile
├── views/           # Page components
│   ├── HomeView.vue           # Main activity list
│   ├── RegisteredView.vue     # User's registered activities
│   ├── ProfileView.vue        # User profile and settings
│   ├── ActivityDetailView.vue # Activity details
│   ├── LogView.vue            # Developer logs
│   └── AboutView.vue          # App information
├── App.vue          # Root component
└── main.ts          # Entry point
```

## Key Features & Implementation

### 1. Authentication (`stores/user.ts`)
- Uses Tauri commands (`login`, `logout`, `get_login_status`) to communicate with the Rust backend.
- Supports auto-login using encrypted credentials stored securely on the device.
- Manages user session state (`logged_in`, `user` info).

### 2. Activity Management (`stores/activity.ts`)
- Fetches activity lists: Recommended, All (Unended), Registered, Participated.
- Handles activity filtering (keyword, module, department, time).
- **Data Structure**: The `Activity` interface maps the JSON response from the backend. Note that the backend returns a flattened JSON structure where dynamic fields (like `pic`, `placeInfo`) are at the root level, which are handled by the frontend interface.

### 3. UI/UX
- **Home View**: Implements pull-to-refresh (restricted to top of page to prevent accidental triggers), collapsible sections for Recommended and All activities.
- **Registered View**: Tabbed interface for "Registered/Ended" and "Participated/Finished" activities. Supports swipe gestures.
- **Activity Card**: Displays activity cover image, status tags, time, location, and organizer. Handles missing data gracefully with placeholders.
- **Safe Area**: Adapts to mobile notches and safe areas using CSS environment variables (`safe-area-inset-top`, etc.).

### 4. Developer Mode & Logs
- **Activation**: Tap the user card in Profile View 7 times to enable Developer Mode.
- **Logging**: Captures app events and errors. Logs can be viewed in-app or saved to the local device (Downloads folder) using `@tauri-apps/plugin-fs`.

## Backend Integration

The frontend communicates with the Rust backend via Tauri's `invoke` command.
- **Commands**: Defined in `src-tauri/src/lib.rs`.
- **Models**: Data structures shared (conceptually) between Rust (`src-tauri/src/rustustc/young/model.rs`) and TypeScript interfaces.

## Build & Deploy

- **Development**: `pnpm tauri dev`
- **Build**: `pnpm tauri build` (generates APK for Android, etc.)
- **CI/CD**: GitHub Actions workflow (`.github/workflows/build.yml`) handles automated builds and releases.

## Future Improvements

- **Offline Mode**: Cache activity data for offline access.
- **Notifications**: Enhanced push notifications for activity reminders.
- **Calendar Integration**: Export activities to system calendar.
