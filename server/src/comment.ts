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
        line = empty(i, line, true)
        i++
      } else if (state === State.No && line[i] === '/' && line[i + 1] === '/' && line[i - 1] !== '*') {
        // TODO early out here
        state = State.Single
        line = empty(i, line, true)
        i++
      } else if (state === State.Multi && line[i] === '*' && line[i + 1] === '/') {
        state = State.No
        // inefficient, try to aggregate it
        line = empty(i, line, true)
        i++
      }

      if (state === State.Multi || state === State.Single) {
        line = empty(i, line, false)
      }
    }
    if (state === State.Single) state = State.No
    return [state, line]
  }

  function empty(i: number, line: string, twice: boolean): string {
    line = line.substr(0, i) + ' ' + line.substr(i + 1)
    if (twice) {
      i++
      line = line.substr(0, i) + ' ' + line.substr(i + 1)
    }
    return line
  }
}