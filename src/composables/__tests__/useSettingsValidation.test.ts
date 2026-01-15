import { describe, it, expect, beforeEach } from 'vitest'
import { useSettingsValidation } from '../useSettingsValidation'
import type { AppConfig } from '@/types'

function createMockConfig(overrides: Partial<AppConfig> = {}): AppConfig {
  return {
    hotkey: 'Ctrl+Shift+V',
    aiProvider: 'Ollama',
    ollamaBaseUrl: 'http://localhost:11434',
    openaiBaseUrl: 'https://api.openai.com/v1',
    modelName: 'llama3.2',
    theme: 'system',
    pinnedRuleIds: [],
    customRules: [],
    enableAIRuleLearning: false,
    ...overrides,
  }
}

describe('useSettingsValidation', () => {
  let validation: ReturnType<typeof useSettingsValidation>

  beforeEach(() => {
    validation = useSettingsValidation()
  })

  describe('validateAll', () => {
    it('should return true for valid Ollama config', () => {
      const config = createMockConfig()
      const result = validation.validateAll(config, '')
      expect(result).toBe(true)
      expect(validation.errors.value).toEqual({})
    })

    it('should return true for valid OpenAI config', () => {
      const config = createMockConfig({
        aiProvider: 'OpenAI',
        openaiBaseUrl: 'https://api.openai.com/v1',
      })
      const result = validation.validateAll(config, 'sk-test-key')
      expect(result).toBe(true)
    })

    it('should fail when hotkey is empty', () => {
      const config = createMockConfig({ hotkey: '' })
      const result = validation.validateAll(config, '')
      expect(result).toBe(false)
      expect(validation.errors.value.general?.hotkey).toBe('热键不能为空')
    })

    it('should fail when hotkey is whitespace only', () => {
      const config = createMockConfig({ hotkey: '   ' })
      const result = validation.validateAll(config, '')
      expect(result).toBe(false)
      expect(validation.errors.value.general?.hotkey).toBe('热键不能为空')
    })

    it('should fail when Ollama URL is empty', () => {
      const config = createMockConfig({ ollamaBaseUrl: '' })
      const result = validation.validateAll(config, '')
      expect(result).toBe(false)
      expect(validation.errors.value.ai?.baseUrl).toBe('URL 不能为空')
    })

    it('should fail when OpenAI URL is empty', () => {
      const config = createMockConfig({
        aiProvider: 'OpenAI',
        openaiBaseUrl: '',
      })
      const result = validation.validateAll(config, 'sk-key')
      expect(result).toBe(false)
      expect(validation.errors.value.ai?.baseUrl).toBe('URL 不能为空')
    })

    it('should fail when OpenAI URL is not HTTPS', () => {
      const config = createMockConfig({
        aiProvider: 'OpenAI',
        openaiBaseUrl: 'http://api.openai.com/v1',
      })
      const result = validation.validateAll(config, 'sk-key')
      expect(result).toBe(false)
      expect(validation.errors.value.ai?.baseUrl).toBe('OpenAI API 必须使用 HTTPS')
    })

    it('should fail when OpenAI API key is empty', () => {
      const config = createMockConfig({
        aiProvider: 'OpenAI',
        openaiBaseUrl: 'https://api.openai.com/v1',
      })
      const result = validation.validateAll(config, '')
      expect(result).toBe(false)
      expect(validation.errors.value.ai?.apiKey).toBe('OpenAI 需要 API Key')
    })

    it('should not require API key for Ollama', () => {
      const config = createMockConfig({ aiProvider: 'Ollama' })
      const result = validation.validateAll(config, '')
      expect(result).toBe(true)
      expect(validation.errors.value.ai?.apiKey).toBeUndefined()
    })

    it('should fail when model name is empty', () => {
      const config = createMockConfig({ modelName: '' })
      const result = validation.validateAll(config, '')
      expect(result).toBe(false)
      expect(validation.errors.value.ai?.model).toBe('模型名称不能为空')
    })

    it('should collect multiple errors', () => {
      const config = createMockConfig({
        hotkey: '',
        aiProvider: 'OpenAI',
        openaiBaseUrl: 'http://bad-url',
        modelName: '',
      })
      const result = validation.validateAll(config, '')
      expect(result).toBe(false)
      expect(validation.errors.value.general?.hotkey).toBeDefined()
      expect(validation.errors.value.ai?.baseUrl).toBeDefined()
      expect(validation.errors.value.ai?.apiKey).toBeDefined()
      expect(validation.errors.value.ai?.model).toBeDefined()
    })
  })

  describe('setSaveError', () => {
    it('should set save error message', () => {
      validation.setSaveError('保存失败')
      expect(validation.errors.value.save).toBe('保存失败')
    })
  })

  describe('clearErrors', () => {
    it('should clear all errors', () => {
      const config = createMockConfig({ hotkey: '', modelName: '' })
      validation.validateAll(config, '')
      expect(Object.keys(validation.errors.value).length).toBeGreaterThan(0)

      validation.clearErrors()
      expect(validation.errors.value).toEqual({})
    })
  })

  describe('getFlatErrors', () => {
    it('should flatten nested errors', () => {
      const config = createMockConfig({
        hotkey: '',
        aiProvider: 'OpenAI',
        openaiBaseUrl: 'http://bad',
        modelName: '',
      })
      validation.validateAll(config, '')
      validation.setSaveError('保存错误')

      const flat = validation.getFlatErrors()
      expect(flat.hotkey).toBe('热键不能为空')
      expect(flat.baseUrl).toBe('OpenAI API 必须使用 HTTPS')
      expect(flat.apiKey).toBe('OpenAI 需要 API Key')
      expect(flat.model).toBe('模型名称不能为空')
      expect(flat.save).toBe('保存错误')
    })

    it('should return empty object when no errors', () => {
      const flat = validation.getFlatErrors()
      expect(flat).toEqual({})
    })
  })
})
