export class Config {
  public readonly minecraftPath: string
  public readonly glslangPath: string

  constructor(mcPath: string, glslangPath: string) {
    this.minecraftPath = mcPath
    this.glslangPath = glslangPath || 'glslangValidator'
  }
}