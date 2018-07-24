import { connection } from './server'

export function postError(e: Error) {
    connection.window.showErrorMessage(e.message)
    console.log(e)
}

export const formatURI = (uri: string) => uri.replace(/^file:\/\//, '').replace(/^(?:\/)c%3A/, 'C:').replace(/\\/g, '/')