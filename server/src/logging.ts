import { Logger } from 'ts-log-debug'

const defaultOpts = {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['debug', 'info', 'warn', 'error']
}

export const glslProviderLog = new Logger('glslang')
glslProviderLog.appenders.set('std-log', defaultOpts)

export const linterLog = new Logger('linter')
linterLog.appenders.set('std-log', defaultOpts)

export const completionLog = new Logger('completion')
completionLog.appenders.set('std-log', defaultOpts)

export const serverLog = new Logger('server')
serverLog.appenders.set('std-log', defaultOpts)

export const linkLog = new Logger('links')
linkLog.appenders.set('std-log', defaultOpts)

export const uriLog = new Logger('uri')
uriLog.appenders.set('std-log', defaultOpts)

// not added to loggers as this should always log changes
export const configLog = new Logger('config')
configLog.appenders.set('std-log', defaultOpts)

export const loggers = [glslProviderLog, linterLog, completionLog, serverLog, linkLog, uriLog]