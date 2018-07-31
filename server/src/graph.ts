type Node = {
  parents: Map<string, Node>
  children: Map<string, Node>
}

export class Graph {
  public nodes: Map<string, Node> = new Map()

  public hasParents(uri: string): boolean {
    return this.nodes.has(uri) ? this.nodes.get(uri).parents.size > 0 : false
  }

  public setParent(uri: string, parent: string) {
    const par: Node = this.nodes.has(parent) ? this.nodes.get(parent) : {parents: new Map(), children: new Map()}
    if (this.nodes.has(uri)) {
      const node = this.nodes.get(uri)
      node.parents.set(parent, par)
      par.children.set(uri, node)
    } else {
      const node: Node = {parents: new Map([par].map(p => [parent, p]) as [string, Node][]), children: new Map()}
      par.children.set(uri, node)
      this.nodes.set(uri, node)
    }
    this.nodes.set(parent, par)
  }
}