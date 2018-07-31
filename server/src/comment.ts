export namespace Comment {
  export enum State {
    No,
    Single,
    Multi
  }

  export function update(line: string, state: State): [State, string] {
    for (let i = 0; i < line.length; i++) {
      if (state === State.No && line[i] === '/' && line[i + 1] === '*') {
        state = State.Multi
      } else if (state === State.No && line[i] === '/' && line[i + 1] === '/') {
        state = State.Single
      } else if (state === State.Multi && line[i] === '*' && line[i + 1] === '/' && line[i - 1] !== '/') {
        state = State.No
        // inefficient, try to aggregate it
        line = empty(i, line)
        i++
        line = empty(i, line)
      }
      // inefficient, try to aggregate it
      if (state === State.Single || state === State.Multi) {
        line = empty(i, line)
        i++
        line = empty(i, line)
      }
    }
    if (state === State.Single) state = State.No
    return [state, line]
  }

  function empty(i: number, line: string): string {
    return line.substr(0, i) + ' ' + line.substr(i + 1)
  }
}