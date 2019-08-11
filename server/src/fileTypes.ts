
export type ShaderFileExtension = '.fsh' | '.gsh' | '.vsh'
export type ShaderFileType = 'frag' | 'geom' | 'vert'

export const extensionMap = new Map<ShaderFileExtension, ShaderFileType>([
  ['.fsh', 'frag'],
  ['.gsh', 'geom'],
  ['.vsh', 'vert'],
])