import { Logger } from 'ts-log-debug'

export const glslProviderLog = new Logger('glslangProvider')
glslProviderLog.appenders.set('std-log', {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['info', 'warn', 'error']
})

export const linterLog = new Logger('glslangProvider')
linterLog.appenders.set('std-log', {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['info', 'warn', 'error']
})

export const completionLog = new Logger('glslangProvider')
completionLog.appenders.set('std-log', {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['info', 'warn', 'error']
})

export const serverLog = new Logger('glslangProvider')
serverLog.appenders.set('std-log', {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['info', 'warn', 'error']
})

export const linkLog = new Logger('glslangProvider')
linkLog.appenders.set('std-log', {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['info', 'warn', 'error']
})

export const uriLog = new Logger('glslangProvider')
uriLog.appenders.set('std-log', {
  type: 'stdout',
  layout: {type: 'basic'},
  levels: ['info', 'warn', 'error']
})
