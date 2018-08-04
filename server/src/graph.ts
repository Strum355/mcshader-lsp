// can you imagine that some people out there would import a whole library just for this?
export type Pair<T, S> = {
  first: T,
  second: S
}

export type Node = {
  parents: Map<string, Pair<number, Node>>
  children: Map<string, Node>
}

export class Graph {
  public nodes: Map<string, Node> = new Map()

  public hasParents(uri: string): boolean {
    return this.nodes.has(uri) ? this.nodes.get(uri).parents.size > 0 : false
  }

  public get(uri: string): Node {
    if (!this.nodes.has(uri)) this.nodes.set(uri, {parents: new Map(), children: new Map()})
    return this.nodes.get(uri)
  }

  public setParent(uri: string, parent: string, lineNum: number) {
    const par: Node = this.nodes.has(parent) ? this.nodes.get(parent) : {parents: new Map(), children: new Map()}
    if (this.nodes.has(uri)) {
      const node = this.nodes.get(uri)
      node.parents.set(parent, {first: lineNum, second: par})
      par.children.set(uri, node)
    } else {
      const node: Node = {
        parents: new Map([par].map(p => [parent, {first: lineNum, second: p}]) as [string, Pair<number, Node>][]),
        children: new Map()
      }

      par.children.set(uri, node)
      this.nodes.set(uri, node)
    }
    this.nodes.set(parent, par)
  }
}