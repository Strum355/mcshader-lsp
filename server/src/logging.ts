import { CategoryServiceFactory, CategoryConfiguration, LogLevel, Category } from 'typescript-logging'

CategoryServiceFactory.setDefaultConfiguration(new CategoryConfiguration(LogLevel.Debug))

export const linterLog = new Category('linter')
export const completionLog = new Category('completion')
export const serverLog = new Category('server')