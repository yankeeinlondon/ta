// TypeScript file with Unicode characters (emoji, Chinese, RTL text)

function greetInternational(name: string): string {
    // Emoji
    const emoji: number = "Hello! 👋 🌍"; // Error: string assigned to number

    // Chinese characters
    const chinese: boolean = "你好世界"; // Error: string assigned to boolean

    // RTL text (Arabic)
    const arabic: number = "مرحبا بك"; // Error: string assigned to number

    return `${name} says hello`;
}
