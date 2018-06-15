import { join } from 'path'

export class Config {
  public minecraftPath: string
  public glslangValidatorPath: string

  constructor(mcPath: string, glslangPath: string) {
    this.minecraftPath = join(mcPath, 'shaderpacks')
    this.glslangValidatorPath = glslangPath || 'glslangValidator'
  }
}