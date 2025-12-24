// Basic TypeScript file with a single type error
function greet(name: string): string {
    const userId: number = "123"; // Type error: string assigned to number
    return `Hello, ${name}!`;
}
