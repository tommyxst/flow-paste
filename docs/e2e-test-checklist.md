# FlowPaste MVP E2E Test Checklist

## Pre-requisites
- [ ] Ollama installed and running (`ollama serve`)
- [ ] Model downloaded (`ollama pull llama3.2`)
- [ ] Application built (`npm run tauri:build`)

## Core Flow Tests

### 1. Application Launch
- [ ] App starts without errors
- [ ] Window appears (floating panel style)
- [ ] DevTools opens in debug mode

### 2. Global Hotkey
- [ ] Default hotkey `Ctrl+Shift+V` triggers panel
- [ ] Panel shows/hides on hotkey toggle
- [ ] Hotkey works from any application

### 3. Clipboard Reading
- [ ] Copy text in another app
- [ ] Trigger hotkey - panel shows copied text
- [ ] Privacy scan runs (PII indicator updates)

### 4. Rule Processing
- [ ] Click "格式化 JSON" on JSON content
- [ ] Click "行排序" on multi-line text
- [ ] Click "去除空格" on text with spaces
- [ ] Result displays correctly
- [ ] Content pastes to target app

### 5. AI Processing (Ollama)
- [ ] Type custom prompt in input
- [ ] Press Enter to send
- [ ] Streaming output displays
- [ ] Final result shows
- [ ] Cancel button works during streaming

### 6. Settings Panel
- [ ] Press `Ctrl+,` to open settings
- [ ] Change hotkey - verify it works
- [ ] Switch AI provider (Ollama/OpenAI)
- [ ] Test connection button works
- [ ] Save settings persists after restart

### 7. Privacy Protection
- [ ] Copy text with phone number
- [ ] Verify PII indicator shows "masked"
- [ ] AI processes with masked content
- [ ] Result restores original PII

### 8. Error Handling
- [ ] Disconnect Ollama - verify error message
- [ ] Invalid API key - verify error message
- [ ] Retry button works

## Edge Cases
- [ ] Empty clipboard - graceful handling
- [ ] Very long text (>10KB) - no crash
- [ ] Special characters - correct encoding
- [ ] Multiple rapid hotkey presses - stable

## Performance
- [ ] Panel appears within 200ms of hotkey
- [ ] Rule processing < 50ms
- [ ] No memory leaks after 10+ operations

---
**Test Date**: ___________
**Tester**: ___________
**Build Version**: 0.1.0
**Result**: [ ] PASS / [ ] FAIL
