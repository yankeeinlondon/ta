/**
 * Creates a new user in the system
 * @param name The user's full name
 * @param email The user's email address
 * @returns The newly created user object
 */
export function createUser(name: string, email: string): User {
    return { id: 1, name, email };
}

/**
 * Represents a user in the system
 */
export interface User {
    id: number;
    name: string;
    email: string;
}
