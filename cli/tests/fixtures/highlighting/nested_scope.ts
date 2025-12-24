// TypeScript file with error in deeply nested scope

namespace MyApp {
    export class UserService {
        getUserById(id: string): User {
            const userId: number = id; // Error: string assigned to number
            return { id: userId, name: "Test" };
        }
    }
}

interface User {
    id: number;
    name: string;
}
