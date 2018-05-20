declare global {
  interface String {
    leftTrim: () => string
  }
}

String.prototype.leftTrim = function(): string {
  return this.replace(/^\s+/,'')
}

export {}