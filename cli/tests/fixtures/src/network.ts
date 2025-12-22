// Network utilities with cross-file dependencies
import { NetworkConfig, HttpMethod, RequestOptions } from './types';
import { Logger, DEFAULT_TIMEOUT, MAX_RETRIES } from './utils';

export class NetworkClient {
    private config: NetworkConfig;
    private logger: Logger;

    constructor(config: NetworkConfig) {
        this.config = config;
        this.logger = new Logger('NetworkClient');
    }

    async request<T>(url: string, options: RequestOptions): Promise<T> {
        const fullUrl = `${this.config.baseUrl}${url}`;
        const timeout = options.timeout || this.config.timeout || DEFAULT_TIMEOUT;

        this.logger.log(`${options.method} ${fullUrl}`);

        try {
            const response = await this.executeRequest(fullUrl, options, timeout);
            return response as T;
        } catch (error) {
            this.logger.error('Request failed', error as Error);
            throw error;
        }
    }

    private async executeRequest(
        url: string,
        options: RequestOptions,
        timeout: number,
        retryCount: number = 0
    ): Promise<any> {
        // Simulated fetch implementation
        if (retryCount >= (this.config.retries || MAX_RETRIES)) {
            throw new Error('Max retries exceeded');
        }

        // Would normally use fetch() here
        return { success: true };
    }

    async get<T>(url: string): Promise<T> {
        return this.request<T>(url, { method: HttpMethod.GET });
    }

    async post<T>(url: string, body: any): Promise<T> {
        return this.request<T>(url, {
            method: HttpMethod.POST,
            body,
        });
    }
}

export function createNetworkClient(baseUrl: string): NetworkClient {
    return new NetworkClient({
        baseUrl,
        timeout: DEFAULT_TIMEOUT,
        retries: MAX_RETRIES,
    });
}
