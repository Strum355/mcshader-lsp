import { join } from 'path'

export class Config {
  public readonly minecraftPath: string
  public readonly glslangPath: string

  constructor(mcPath: string, glslangPath: string) {
    this.minecraftPath = join(mcPath, 'shaderpacks')
    this.glslangPath = glslangPath || 'glslangValidator'
  }
}