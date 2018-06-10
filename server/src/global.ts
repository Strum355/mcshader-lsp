declare global {
  interface String {
    leftTrim: () => string
    rightTrim: () => string
  }
}

String.prototype.leftTrim = function(): string {
  return this.replace(/^\s+/,'')
}

String.prototype.rightTrim = function(): string {
  return this.replace(/\s+$/, '')
}

export {}