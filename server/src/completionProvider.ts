import { CompletionItem, CompletionItemKind } from 'vscode-languageserver'

const value = CompletionItemKind.Value

export const completions: CompletionItem[] = [
  {
    label: 'heldItemId',
    detail: '<int> held item ID (main hand)'
  },
  {
    label: 'heldBlockLightValue',
    detail: '<int> held item light value (main hand)'
  },
  {
    label: 'heldItemId2',
    detail: '<int> held item ID (off hand)'
  },
  {
    label: 'heldBlockLightValue2',
    detail: '<int> held item light value (off hand)'
  },
  {
    label: 'fogMode',
    detail: '<int> GL_LINEAR, GL_EXP or GL_EXP2'
  },
  {
    label: 'fogColor',
    detail: '<vec3> r, g, b'
  },
  {
    label: 'skyColor',
    detail: '<vec3> r, g, b'
  },
  {
    label: 'worldTime',
    detail: '<int> <ticks> = worldTicks % 24000'
  },
  {
    label: 'worldDay',
    detail: '<int> <days> = worldTicks / 24000'
  },
  {
    label: 'moonPhase',
    detail: '<int> 0-7'
  },
  {
    label: 'frameCounter',
    detail: '<int> Frame index (0 to 720719, then resets to 0)'
  },
  {
    label: 'frameTime',
    detail: '<float> last frame time, seconds'
  },
  {
    label: 'frameTimeCounter',
    detail: '<float> run time, seconds (resets to 0 after 3600s)'
  },
  {
    label: 'sunAngle',
    detail: '<float> 0.0-1.0'
  },
  {
    label: 'shadowAngle',
    detail: '<float> 0.0-1.0'
  },
  {
    label: 'rainStrength',
    detail: '<float> 0.0-1.0'
  },
  {
    label: 'aspectRatio',
    detail: '<float> viewWidth / viewHeight'
  },
  {
    label: 'viewWidth',
    detail: '<float> viewWidth'
  },
  {
    label: 'viewHeight',
    detail: '<float> viewHeight'
  },
  {
    label: 'near',
    detail: '<float> near viewing plane distance'
  },
  {
    label: 'far',
    detail: '<float> far viewing plane distance'
  },
  {
    label: 'sunPosition',
    detail: '<vec3> sun position in eye space'
  },
  {
    label: 'moonPosition',
    detail: '<vec3> moon position in eye space'
  },
  {
    label: 'shadowLightPosition',
    detail: '<vec3> shadow light (sun or moon) position in eye space'
  },
  {
    label: 'upPosition',
    detail: '<vec3> direction up'
  },
  {
    label: 'cameraPosition',
    detail: '<vec3> camera position in world space'
  },
  {
    label: 'previousCameraPosition',
    detail: '<vec3> last frame cameraPosition'
  },
  {
    label: 'gbufferModelView',
    detail: '<mat4> modelview matrix after setting up the camera transformations'
  },
  {
    label: 'gbufferModelViewInverse',
    detail: '<mat4> inverse gbufferModelView'
  },
  {
    label: 'gbufferPreviousModelView',
    detail: '<mat4> last frame gbufferModelView'
  },
  {
    label: 'gbufferProjection',
    detail: '<mat4> projection matrix when the gbuffers were generated'
  },
  {
    label: 'gbufferProjectionInverse',
    detail: '<mat4> inverse gbufferProjection'
  },
  {
    label: 'gbufferPreviousProjection',
    detail: '<mat4> last frame gbufferProjection'
  },
  {
    label: 'shadowProjection',
    detail: '<mat4> projection matrix when the shadow map was generated'
  },
  {
    label: 'shadowProjectionInverse',
    detail: '<mat4> inverse shadowProjection'
  },
  {
    label: 'shadowModelView',
    detail: '<mat4> modelview matrix when the shadow map was generated'
  },
  {
    label: 'shadowModelViewInverse',
    detail: '<mat4> inverse shadowModelView'
  },
  {
    label: 'wetness',
    detail: '<float> rainStrength smoothed with wetnessHalfLife or drynessHalfLife'
  },
  {
    label: 'eyeAltitude',
    detail: '<float> view entity Y position'
  },
  {
    label: 'eyeBrightness',
    detail: '<ivec2> x = block brightness, y = sky brightness, light 0-15 = brightness 0-240'
  },
  {
    label: 'eyeBrightnessSmooth',
    detail: '<ivec2> eyeBrightness smoothed with eyeBrightnessHalflife'
  },
  {
    label: 'terrainTextureSize',
    detail: '<ivec2> not used'
  },
  {
    label: 'terrainIconSize',
    detail: '<int> not used'
  },
  {
    label: 'isEyeInWater',
    detail: '<int> 1 = camera is in water, 2 = camera is in lava'
  },
  {
    label: 'nightVision',
    detail: '<float> night vision (0.0-1.0)'
  },
  {
    label: 'blindness',
    detail: '<float> blindness (0.0-1.0)'
  },
  {
    label: 'screenBrightness',
    detail: '<float> screen brightness (0.0-1.0)'
  },
  {
    label: 'hideGUI',
    detail: '<int> GUI is hidden'
  },
  {
    label: 'centerDepthSmooth',
    detail: '<float> centerDepth smoothed with centerDepthSmoothHalflife'
  },
  {
    label: 'atlasSize',
    detail: '<ivec2> texture atlas size (only set when the atlas texture is bound)'
  },
  {
    label: 'entityColor',
    detail: '<vec4> entity color multiplier (entity hurt, creeper flashing when exploding)'
  },
  {
    label: 'entityId',
    detail: '<int> entity ID'
  },
  {
    label: 'blockEntityId',
    detail: '<int> block entity ID (block ID for the tile entity)'
  },
  {
    label: 'blendFunc',
    detail: '<ivec4> blend function (srcRGB, dstRGB, srcAlpha, dstAlpha)'
  },
  {
    label: 'texture',
    detail: '<sampler2D>'
  },
  {
    label: 'lightmap',
    detail: '<sampler2D>'
  },
  {
    label: 'normals',
    detail: '<sampler2D>'
  },
  {
    label: 'specular',
    detail: '<sampler2D>'
  },
  {
    label: 'shadow',
    detail: '<sampler2D> waterShadowEnabled ? 5 : 4'
  },
  {
    label: 'watershadow',
    detail: '<sampler2D>'
  },
  {
    label: 'shadowtex0',
    detail: '<sampler2D>'
  },
  {
    label: 'shadowtex1',
    detail: '<sampler2D>'
  },
  {
    label: 'depthtex0',
    detail: '<sampler2D>'
  },
  {
    label: 'gaux1',
    detail: '<sampler2D> 7 <custom texture or output from deferred programs>'
  },
  {
    label: 'gaux2',
    detail: '<sampler2D> 8 <custom texture or output from deferred programs>'
  },
  {
    label: 'gaux3',
    detail: '<sampler2D> 9 <custom texture or output from deferred programs>'
  },
  {
    label: 'gaux4',
    detail: '<sampler2D> 10 <custom texture or output from deferred programs>'
  },
  {
    label: 'depthtex1',
    detail: '<sampler2D>'
  },
  {
    label: 'shadowcolor',
    detail: '<sampler2D>'
  },
  {
    label: 'shadowcolor0',
    detail: '<sampler2D>'
  },
  {
    label: 'shadowcolor1',
    detail: '<sampler2D>'
  },
  {
    label: 'noisetex',
    detail: '<sampler2D>'
  },
  {
    label: 'tex',
    detail: '<sampler2D>'
  },
  {
    label: 'gcolor',
    detail: '<sampler2D>'
  },
  {
    label: 'gdepth',
    detail: '<sampler2D>'
  },
  {
    label: 'gnormal',
    detail: '<sampler2D>'
  },
  {
    label: 'composite',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex0',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex1',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex2',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex3',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex4',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex5',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex6',
    detail: '<sampler2D>'
  },
  {
    label: 'colortex7',
    detail: '<sampler2D>'
  },
  {
    label: 'gdepthtex',
    detail: '<sampler2D>'
  },
  {
    label: 'depthtex2',
    detail: '<sampler2D>'
  },
  {
    label: 'depthtex3',
    detail: '<type> depthBuffers = 4'
  }
]

for (let i = 1; i < completions.length + 1; i++) {
  completions[i - 1].data = i
  completions[i - 1].kind = value
}