export class Config {
  public readonly shaderpacksPath: string
  public readonly glslangPath: string

  constructor(shaderpacksPath: string, glslangPath: string) {
    this.shaderpacksPath = shaderpacksPath
    this.glslangPath = glslangPath || 'glslangValidator'
  }
}