import { connection } from './server'

export function postError(e: Error) {
    connection.window.showErrorMessage(e.message)
    console.log(e)
}