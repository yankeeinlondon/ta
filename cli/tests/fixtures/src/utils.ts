// Utility functions

export function formatDate(date: Date): string {
    return date.toISOString().split('T')[0];
}

export function validateEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

export function debounce<T extends (...args: any[]) => any>(
    func: T,
    wait: number
): (...args: Parameters<T>) => void {
    let timeout: NodeJS.Timeout | null = null;

    return function(...args: Parameters<T>) {
        if (timeout) clearTimeout(timeout);
        timeout = setTimeout(() => func(...args), wait);
    };
}

export class Logger {
    private prefix: string;

    constructor(prefix: string = 'LOG') {
        this.prefix = prefix;
    }

    log(message: string): void {
        console.log(`[${this.prefix}] ${message}`);
    }

    error(message: string, error?: Error): void {
        console.error(`[${this.prefix}] ERROR: ${message}`, error);
    }

    debug(message: string, data?: any): void {
        console.debug(`[${this.prefix}] DEBUG: ${message}`, data);
    }
}

export const DEFAULT_TIMEOUT = 5000;
export const MAX_RETRIES = 3;
