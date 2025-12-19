# Tasks Document - FlowPaste (MVP v1.0)

## Phase 1: Project Scaffolding

- [x] 1. Initialize Tauri + Vue 3 project
  - Files: `package.json`, `vite.config.ts`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
  - Initialize Tauri 2.x project with Vue 3 + TypeScript template
  - Configure TailwindCSS and essential plugins
  - Set up project structure according to design document
  - _Leverage: Tauri CLI, Vite, TailwindCSS_
  - _Requirements: REQ-01 (foundation)_

- [x] 2. Configure Tauri Window Settings (Spotlight Style)
  - File: `src-tauri/tauri.conf.json`
  - Configure transparent, frameless window
  - Set up window positioning and always-on-top behavior
  - _Leverage: Tauri window configuration_
  - _Requirements: REQ-17, UI Design 4.1_

- [x] 3. Define TypeScript interfaces
  - File: `src/types/index.ts`
  - Define ActionChip, PrivacyStatus, PIIItem, Rule, AIConfig interfaces
  - Add IPC event payload types following design.md IPC Event Specification
  - Ensure type compatibility with Rust structs via Tauri IPC
  - _Leverage: design.md Data Models section, IPC Event Specification_
  - _Requirements: All (type foundation)_

- [x] 4. Set up Logging Infrastructure
  - Files: `src/utils/logger.ts`, `src/lib/tauri.ts`
  - Frontend logger with module-based logging
  - Tauri IPC wrapper with debug logging
  - Backend uses env_logger (configured in lib.rs)
  - _Leverage: env_logger crate (Rust), custom logger (Vue)_
  - _Requirements: Security (No Content Logging)_

---

## Phase 2: Backend Core Modules (Rust)

- [ ] 5. Implement Clipboard Module
  - Files: `src-tauri/src/clipboard/mod.rs`, `src-tauri/src/clipboard/manager.rs`
  - Implement read_clipboard() and write_clipboard() using arboard crate
  - Handle non-text clipboard content detection
  - Create Tauri command wrappers
  - _Leverage: arboard crate_
  - _Requirements: REQ-02, REQ-03, REQ-14_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with systems programming expertise | Task: Implement clipboard module with read_clipboard() returning Result<String, ClipboardError>, write_clipboard(content: &str), detect_content_type() for non-text handling, expose as Tauri commands | Restrictions: Handle clipboard access errors gracefully, do not panic on failure, support both Windows and macOS, return clear error for non-text content | _Leverage: arboard crate documentation | Success: Can read/write clipboard via Tauri commands, proper error handling, non-text detection works | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 6. Implement OS Permission Checker
  - File: `src-tauri/src/permissions/mod.rs`
  - Check macOS Accessibility permission status
  - Provide permission request guidance
  - _Leverage: Tauri APIs, platform-specific crates_
  - _Requirements: REQ-15_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with macOS expertise | Task: Implement permission checker - check_accessibility_permission() for macOS using accessibility-sys or similar, return PermissionStatus enum, provide open_system_preferences() to guide user | Restrictions: Cache permission status, re-check on operation failure, handle Windows (always granted) gracefully | _Leverage: Platform-specific APIs | Success: Permission status correctly detected on macOS, helpful guidance provided | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 7. Implement Config Manager with SQLite
  - Files: `src-tauri/src/config/mod.rs`, `src-tauri/src/config/store.rs`
  - Set up SQLite database for app configuration
  - Implement load_config() and save_config() functions
  - Create config table schema
  - _Leverage: rusqlite crate_
  - _Requirements: REQ-01 (hotkey config), REQ-08, REQ-09 (AI config)_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with database expertise | Task: Implement config manager using rusqlite - create SQLite database in app data dir, config table (key TEXT PRIMARY KEY, value TEXT, updated_at INTEGER), implement load_config/save_config with AppConfig struct | Restrictions: Use Tauri's app data directory, handle database errors properly, use prepared statements | _Leverage: rusqlite crate, Tauri path API | Success: Config persists across app restarts, CRUD operations work correctly | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 8. Implement Keychain Integration
  - File: `src-tauri/src/config/keychain.rs`
  - Integrate with OS keychain (Windows Credential Locker / macOS Keychain)
  - Implement get_api_key() and set_api_key() functions
  - _Leverage: keyring crate_
  - _Requirements: REQ-09 (secure credential storage)_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with security expertise | Task: Implement keychain integration using keyring crate - get_api_key(provider: &str) and set_api_key(provider: &str, key: &str), use "flow-paste" as service name | Restrictions: Never log or expose API keys, handle keychain access errors, support both Windows and macOS | _Leverage: keyring crate documentation | Success: API keys stored securely in OS keychain, retrievable across sessions | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 9. Implement Regex Engine with Built-in Rules
  - Files: `src-tauri/src/regex/mod.rs`, `src-tauri/src/regex/rules.rs`
  - Define Rule struct and built-in rules (remove empty lines, trim whitespace, etc.)
  - Implement apply_rule() and get_builtin_rules() functions
  - _Leverage: regex crate_
  - _Requirements: REQ-05_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Implement regex engine - Rule struct (id, name, description, pattern, replacement, is_builtin), apply_rule(text, rule) using regex crate, get_builtin_rules() returning 5 rules: remove_empty_lines, trim_whitespace, cjk_spacing, to_plain_text, collapse_spaces | Restrictions: Rules must complete within 50ms, handle invalid regex gracefully, use lazy_static for compiled patterns | _Leverage: regex crate | Success: All 5 built-in rules work correctly, performance under 50ms | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [x] 10. Implement Privacy Shield - PII Scanner
  - Files: `src-tauri/src/privacy/mod.rs`, `src-tauri/src/privacy/scanner.rs`, `src-tauri/src/privacy/patterns.rs`
  - Define PII patterns (phone, email, ID card, bank card, IP, API key)
  - Implement scan_pii() returning PIIScanResult
  - _Leverage: regex crate, design.md patterns_
  - _Requirements: REQ-10_
  - _Completed: Implemented 6 PII types with priority-based detection, Luhn check for bank cards, overlap prevention_

- [x] 11. Implement Privacy Shield - Masker and Restorer
  - File: `src-tauri/src/privacy/masker.rs`
  - Implement mask_pii() replacing sensitive data with {{FP_TYPE_ID}} placeholders
  - Implement restore_pii() to rehydrate placeholders
  - _Leverage: Task 10 scanner_
  - _Requirements: REQ-11_
  - _Completed: mask_pii/restore_pii with {{FP_TYPE_ID}} format, MaskMapping for round-trip restoration_

- [x] 12. Implement AI Engine - OpenAI Compatible Client
  - Files: `src-tauri/src/ai/mod.rs`, `src-tauri/src/ai/openai.rs`
  - Implement streaming completion API client
  - Support configurable base_url for OpenAI/DeepSeek/compatible APIs
  - _Leverage: reqwest crate, tokio_
  - _Requirements: REQ-09, REQ-07, REQ-13, REQ-16_
  - _Completed: AiProvider trait, OpenAIProvider with SSE streaming, buffer-based parsing, auth handling_

- [x] 13. Implement AI Engine - Ollama Client
  - File: `src-tauri/src/ai/ollama.rs`
  - Implement Ollama API client with model listing
  - Support local model detection and streaming
  - _Leverage: reqwest crate, Ollama API spec_
  - _Requirements: REQ-08_
  - _Completed: OllamaProvider with /api/tags and /api/generate endpoints, health_check, streaming support_

- [ ] 14. Implement AI Engine - Intent Detection
  - File: `src-tauri/src/ai/intent.rs`
  - Implement detect_intent() to analyze clipboard content
  - Generate ActionChip recommendations based on content type
  - _Leverage: AI clients from Tasks 12-13_
  - _Requirements: REQ-06_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with ML/NLP background | Task: Implement intent detection - detect_intent(text) analyzing content type (JSON, code, prose, table), return Vec<ActionChip> with 1-3 contextual recommendations, use heuristics first (regex for JSON/code detection) then optional AI enhancement | Restrictions: Heuristic detection must be under 10ms, ActionChips include shortcut keys '1','2','3' | _Leverage: regex for pattern matching | Success: Correct content type detection, relevant action recommendations | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 15. Implement Hotkey Module
  - Files: `src-tauri/src/hotkey/mod.rs`, `src-tauri/src/hotkey/manager.rs`
  - Register global hotkey using Tauri global-shortcut plugin
  - Support custom hotkey configuration
  - Handle hotkey conflicts
  - _Leverage: Tauri global-shortcut plugin, Config Manager_
  - _Requirements: REQ-01_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Tauri expertise | Task: Implement hotkey module using tauri-plugin-global-shortcut - register_hotkey(keys) parsing accelerator string, unregister_hotkey(), emit "panel:toggle" event on trigger, load default from config (CommandOrControl+Shift+V), detect and report hotkey conflicts | Restrictions: Handle hotkey conflicts gracefully with user notification, support re-registration on config change, provide tray icon fallback | _Leverage: tauri-plugin-global-shortcut | Success: Global hotkey triggers panel toggle, conflicts detected, custom hotkeys work | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [-] 16. Create Tauri Command Layer
  - File: `src-tauri/src/commands/mod.rs`, `privacy.rs`, `ai.rs`
  - Expose all backend functions as Tauri commands
  - Implement proper error serialization
  - _Leverage: All backend modules_
  - _Requirements: All backend requirements_
  - _Partial: AI commands (send_ai_request, cancel_ai_request, list_local_models, check_ollama_health) and Privacy commands (scan_pii, mask_pii, restore_pii) completed. Remaining: clipboard, config commands_

---

## Phase 3: Frontend Components (Vue 3)

- [ ] 17. Create Pinia Store
  - File: `src/stores/app.ts`
  - Implement useAppStore with all state and actions from design
  - Connect to Tauri IPC for backend calls
  - _Leverage: Pinia, design.md State Management section_
  - _Requirements: All frontend requirements_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer with Pinia expertise | Task: Implement useAppStore following design.md - state (isVisible, panelMode, clipboardContent, processedContent, streamingContent, actionChips, privacyStatus, maskedMapping), actions (showPanel, hidePanel, processWithRule, processWithAI, cancelAI, confirmPaste) calling Tauri invoke | Restrictions: Use Composition API setup syntax, handle async errors properly | _Leverage: Pinia, @tauri-apps/api | Success: Store manages all app state, Tauri commands called correctly | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [-] 18. Create Tauri IPC Wrapper
  - File: `src/lib/tauri.ts`
  - Create typed wrappers for all Tauri invoke calls
  - Set up event listeners for streaming responses (ai:chunk, ai:done, ai:error)
  - _Leverage: @tauri-apps/api, TypeScript interfaces, design.md IPC Event Specification_
  - _Requirements: All (IPC foundation)_
  - _Partial: AI and Privacy commands added (scanPii, maskPii, restorePii, listLocalModels, checkOllamaHealth, sendAiRequest, cancelAiRequest). Remaining: clipboard, config commands_

- [-] 19. Implement FloatingPanel Component
  - File: `src/components/FloatingPanel.vue`
  - Create Spotlight-style floating panel container
  - Handle panel visibility and keyboard navigation
  - _Leverage: TailwindCSS, Pinia store, design.md Window Behavior Specification_
  - _Requirements: REQ-01, REQ-17, UI Design 4.1_
  - _Partial: AI streaming integration, command input, preview, ModelBadge, PrivacyIndicator added. Remaining: auto-hide on blur, focus management refinement_

- [ ] 20. Implement CommandInput Component
  - File: `src/components/CommandInput.vue`
  - Create input field for natural language commands
  - Handle Enter to submit, keyboard shortcuts
  - _Leverage: TailwindCSS_
  - _Requirements: REQ-07, UI Design 4.2_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement CommandInput.vue - borderless input with placeholder "Describe format or choose below...", emit submit on Enter, expose focus method, handle disabled state during processing | Restrictions: Auto-focus on mount, clean minimal styling | _Leverage: TailwindCSS | Success: Input works with proper focus management and submit handling | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 21. Implement Preview Component
  - File: `src/components/Preview.vue`
  - Display clipboard content preview and processed results
  - Support streaming output with typewriter effect
  - Handle large content truncation display
  - _Leverage: shiki for syntax highlighting_
  - _Requirements: REQ-02, REQ-13, UI Design 4.2_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement Preview.vue - props (content, mode: preview|result|streaming, highlight, truncated), show first 3 lines in preview mode (gray text), full content in result mode, typewriter animation in streaming mode, show truncation warning if content was cut, optional syntax highlighting via shiki | Restrictions: Limit preview to 3 lines, handle long content with ellipsis, smooth streaming animation, show truncation notice | _Leverage: shiki, TailwindCSS | Success: All three modes render correctly, streaming looks smooth, truncation shown | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 22. Implement ActionChips Component
  - File: `src/components/ActionChips.vue`
  - Display AI-recommended action buttons
  - Handle selection via click and number keys (1, 2, 3)
  - _Leverage: TailwindCSS_
  - _Requirements: REQ-06, UI Design 4.2_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement ActionChips.vue - props (chips: ActionChip[], selectedIndex), render chips as pill buttons with shortcut prefix [1. Label], emit select on click or number key, highlight selected chip, support keyboard navigation | Restrictions: Max 3 chips, show shortcut keys clearly | _Leverage: TailwindCSS | Success: Chips render correctly, keyboard selection works | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [x] 23. Implement PrivacyIndicator Component
  - File: `src/components/PrivacyIndicator.vue`
  - Display privacy shield status icon
  - Show tooltip with masked item count
  - _Leverage: TailwindCSS_
  - _Requirements: REQ-12, UI Design 4.2_
  - _Completed: Three states (local/cloud-safe/cloud-masked), color-coded badges, tooltip with status description_

- [x] 23a. Implement ModelBadge Component (Added)
  - File: `src/components/ModelBadge.vue`
  - Display current AI provider status (Local/Cloud)
  - Show connection status indicator
  - _Completed: Provider badge with connection indicator, dark mode support_

- [x] 23b. Implement StreamingOutput Component (Added)
  - File: `src/components/StreamingOutput.vue`
  - Display streaming AI response with typewriter effect
  - Support cancel during streaming
  - _Completed: Typewriter animation, cancel button, auto-scroll, cursor indicator_

- [ ] 24. Implement Settings Panel
  - File: `src/components/SettingsPanel.vue` (new)
  - Create settings UI for API configuration
  - Handle hotkey customization
  - URL validation (HTTPS enforcement)
  - _Leverage: Pinia store, Tauri config commands_
  - _Requirements: REQ-01, REQ-08, REQ-09_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement SettingsPanel.vue - form for: hotkey input, AI provider select (openai/ollama), base_url input with HTTPS validation, API key input (password type), model select, save/cancel buttons, validation feedback, test connection button | Restrictions: API key input must be masked, validate URLs (reject http://), test connection before save | _Leverage: useAppStore, Tauri config commands | Success: Settings save correctly, API connection testable, HTTPS enforced | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 25. Create Composables
  - Files: `src/composables/useClipboard.ts`, `useHotkey.ts`, `useAI.ts`
  - Implement reusable composition functions
  - Encapsulate Tauri IPC logic
  - _Leverage: Tauri IPC wrapper, Pinia store_
  - _Requirements: All frontend requirements_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement composables - useClipboard() for read/paste operations, useHotkey() for listening to panel:toggle events, useAI() for AI processing with streaming support and cancellation, all using Tauri IPC wrapper | Restrictions: Properly handle component lifecycle (onMounted/onUnmounted), clean up event listeners | _Leverage: @tauri-apps/api, Pinia | Success: Composables encapsulate logic cleanly, reusable across components | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 26. Assemble App.vue with All Components
  - File: `src/App.vue`
  - Integrate all components into main app
  - Handle global keyboard events and panel lifecycle
  - _Leverage: All components, Pinia store, composables_
  - _Requirements: All UI requirements_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement App.vue - use FloatingPanel as root, integrate CommandInput, Preview, ActionChips, PrivacyIndicator, SettingsPanel, connect to useAppStore, handle global keyboard (Esc to close/cancel, number keys for chips, Enter to confirm), manage panel state transitions | Restrictions: Clean component composition, proper event handling | _Leverage: All components and composables | Success: Full UI workflow works end-to-end | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

---

## Phase 4: Integration & Polish

- [ ] 27. Implement Paste-to-Cursor Functionality
  - File: `src-tauri/src/clipboard/manager.rs` (extend)
  - Implement keyboard simulation for pasting at cursor
  - Support both keyboard simulation and clipboard API
  - Handle permission failures gracefully
  - _Leverage: enigo crate or platform APIs_
  - _Requirements: REQ-03, REQ-15_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with systems programming expertise | Task: Extend clipboard manager - implement paste_to_cursor(text) that writes to clipboard then simulates Ctrl/Cmd+V keystroke using enigo crate, handle platform differences (Windows/macOS), fallback to clipboard-only if simulation fails due to permissions | Restrictions: Must work across different applications, handle focus properly, graceful degradation on permission failure | _Leverage: enigo crate | Success: Text pastes at cursor position in target app, fallback works | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 28. Implement Dark/Light Mode Support
  - Files: `src/App.vue`, `tailwind.config.js`
  - Detect system theme preference
  - Apply dark/light styles throughout UI
  - _Leverage: TailwindCSS dark mode, Tauri theme API_
  - _Requirements: UI Design 4.1_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer | Task: Implement theme support - configure Tailwind darkMode: 'class', detect system preference via window.matchMedia or Tauri theme API, apply dark class to html element, ensure all components use dark: variants | Restrictions: Respect system preference, smooth transition when theme changes | _Leverage: TailwindCSS, prefers-color-scheme | Success: UI correctly follows system theme | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 29. Add Error Handling UI
  - File: `src/components/ErrorDisplay.vue` (new)
  - Create error display component with shake animation
  - Handle retry and dismiss actions
  - _Leverage: TailwindCSS animations_
  - _Requirements: UI Design 4.3_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Vue Developer | Task: Implement ErrorDisplay.vue - props (error: ErrorResponse), show red error message, shake animation on appear, retry button if error.recoverable, dismiss button, emit retry/dismiss events | Restrictions: Non-blocking (user can still close panel) | _Leverage: TailwindCSS animate-shake | Success: Errors display clearly with appropriate actions | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

---

## Phase 5: Testing & Finalization

- [ ] 30. Write Rust Unit Tests
  - Files: `src-tauri/src/*/mod.rs` (test modules)
  - Test PII detection patterns
  - Test regex rules
  - Test mask/restore round-trip
  - Test URL validation (HTTPS enforcement)
  - _Leverage: Rust test framework_
  - _Requirements: All backend requirements_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with Rust expertise | Task: Write unit tests - test_phone_detection, test_email_detection, test_all_pii_types, test_mask_restore_roundtrip, test_builtin_rules, test_config_persistence, test_https_enforcement, test_url_validation | Restrictions: Test edge cases, use test fixtures, maintain test isolation | _Leverage: #[cfg(test)] modules | Success: All tests pass, good coverage of core logic | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 31. Write Vue Component Tests
  - Files: `src/components/__tests__/*.spec.ts`
  - Test component rendering and interactions
  - Test Pinia store actions
  - _Leverage: Vitest, Vue Test Utils_
  - _Requirements: All frontend requirements_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with Vue testing expertise | Task: Write component tests using Vitest + Vue Test Utils - test FloatingPanel visibility toggle and focus-loss hide, CommandInput submit handling, ActionChips keyboard selection, Preview mode switching, store action side effects, cancel functionality | Restrictions: Mock Tauri IPC calls, test user interactions | _Leverage: Vitest, @vue/test-utils | Success: All component tests pass, interactions verified | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 32. End-to-End Integration Test
  - File: `tests/e2e/flow.spec.ts`
  - Test complete user flow: hotkey → select → paste
  - Test AI processing flow with mock
  - Test cancel/timeout scenarios
  - _Leverage: Playwright, Tauri Driver_
  - _Requirements: All requirements_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Automation Engineer | Task: Write E2E tests using Playwright + Tauri - test_basic_flow (trigger hotkey, verify panel, select action, verify paste), test_ai_flow (input command, verify streaming, confirm result), test_privacy_flow (verify PII masking indicator), test_cancel_flow (start AI, cancel, verify state), test_focus_loss (click outside, verify hide) | Restrictions: Use Tauri test driver, handle async properly | _Leverage: @playwright/test, tauri-driver | Success: E2E tests pass, critical flows verified | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 33. Set up CI Pipeline
  - Files: `.github/workflows/ci.yml`
  - Configure GitHub Actions for Rust and Vue tests
  - Add linting and formatting checks
  - _Leverage: GitHub Actions, cargo, npm_
  - _Requirements: All_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Create CI pipeline - GitHub Actions workflow running on push/PR, jobs: rust-test (cargo test, cargo clippy, cargo fmt --check), vue-test (npm test, eslint, prettier --check), build-check (npm run tauri build --debug) | Restrictions: Cache dependencies for speed, fail on warnings | _Leverage: GitHub Actions, actions/cache | Success: CI runs on every PR, catches issues early | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_

- [ ] 34. Final Integration and Build Configuration
  - Files: `package.json`, `src-tauri/tauri.conf.json`
  - Configure production build settings
  - Set up app icons and metadata
  - Test release build on Windows/macOS
  - _Leverage: Tauri build system_
  - _Requirements: All_
  - _Prompt: Implement the task for spec flow-paste, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Finalize build config - set app identifier (com.flowpaste.app), configure icons (1024x1024 source), set version 1.0.0, configure Windows installer and macOS DMG, test npm run tauri build on both platforms | Restrictions: Ensure code signing ready (even if not signed yet), optimize bundle size | _Leverage: Tauri build documentation | Success: Release builds work on Windows and macOS | Before starting: Mark this task as [-] in tasks.md | After completion: Use log-implementation tool to record artifacts, then mark as [x] in tasks.md_
