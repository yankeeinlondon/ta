// TypeScript file with multiple type errors across different scopes

function calculate(a: number, b: string): number {
    const result: string = a + b; // Error 1: number + string assigned to string
    return result; // Error 2: string returned where number expected
}

class Calculator {
    add(x: boolean, y: number): number {
        const sum: string = x + y; // Error 3: boolean + number assigned to string
        return sum; // Error 4: string returned where number expected
    }
}

const invalidAssignment: number = "not a number"; // Error 5: string assigned to number
