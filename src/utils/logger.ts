type LogLevel = 'debug' | 'info' | 'warn' | 'error'

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
}

class Logger {
  private level: LogLevel = import.meta.env.DEV ? 'debug' : 'info'
  private prefix = '[FlowPaste]'

  private shouldLog(level: LogLevel): boolean {
    return LOG_LEVELS[level] >= LOG_LEVELS[this.level]
  }

  private formatMessage(level: LogLevel, module: string, message: string): string {
    const timestamp = new Date().toISOString().slice(11, 23)
    return `${this.prefix} ${timestamp} [${level.toUpperCase()}] [${module}] ${message}`
  }

  debug(module: string, message: string, ...args: unknown[]) {
    if (this.shouldLog('debug')) {
      console.debug(this.formatMessage('debug', module, message), ...args)
    }
  }

  info(module: string, message: string, ...args: unknown[]) {
    if (this.shouldLog('info')) {
      console.info(this.formatMessage('info', module, message), ...args)
    }
  }

  warn(module: string, message: string, ...args: unknown[]) {
    if (this.shouldLog('warn')) {
      console.warn(this.formatMessage('warn', module, message), ...args)
    }
  }

  error(module: string, message: string, ...args: unknown[]) {
    if (this.shouldLog('error')) {
      console.error(this.formatMessage('error', module, message), ...args)
    }
  }

  setLevel(level: LogLevel) {
    this.level = level
  }
}

export const logger = new Logger()

export function createModuleLogger(module: string) {
  return {
    debug: (message: string, ...args: unknown[]) => logger.debug(module, message, ...args),
    info: (message: string, ...args: unknown[]) => logger.info(module, message, ...args),
    warn: (message: string, ...args: unknown[]) => logger.warn(module, message, ...args),
    error: (message: string, ...args: unknown[]) => logger.error(module, message, ...args),
  }
}
