// File with intentional errors for testing
// These are semantic errors that OXC can detect

export function processUser() {
    // Redeclaration error - OXC detects this
    let userId = 1;
    let userId = 2;  // Error: Identifier 'userId' has already been declared

    return userId;
}

export function assignRole() {
    // Another redeclaration error
    const role = 'admin';
    const role = 'user';  // Error: Identifier 'role' has already been declared

    return role;
}

export class DataProcessor {
    processData() {
        // Redeclaration error in method
        var count = 0;
        var count = 1;  // Error: Identifier 'count' has already been declared

        return count;
    }
}
