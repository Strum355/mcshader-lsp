import { Memento } from 'vscode'
import { log } from './log'

export class PersistentState {
  constructor(private readonly state: Memento) {
    const { serverVersion } = this
    log.info('working with state', { serverVersion })
  }

  public get serverVersion(): string | undefined {
    return this.state.get('serverVersion')
  }

  async updateServerVersion(value: string | undefined) {
    await this.state.update('serverVersion', value)
  }
}