// API layer with dependencies on network and types
import { NetworkClient, createNetworkClient } from './network';
import { User, ApiResponse } from './types';
import { validateEmail } from './utils';

export class UserApi {
    private client: NetworkClient;

    constructor(baseUrl: string) {
        this.client = createNetworkClient(baseUrl);
    }

    async getUser(userId: number): Promise<User> {
        const response = await this.client.get<ApiResponse<User>>(
            `/users/${userId}`
        );
        return response.data;
    }

    async createUser(name: string, email: string): Promise<User> {
        if (!validateEmail(email)) {
            throw new Error('Invalid email address');
        }

        const response = await this.client.post<ApiResponse<User>>('/users', {
            name,
            email,
            role: 'user',
        });

        return response.data;
    }

    async updateUser(userId: number, updates: Partial<User>): Promise<User> {
        if (updates.email && !validateEmail(updates.email)) {
            throw new Error('Invalid email address');
        }

        const response = await this.client.post<ApiResponse<User>>(
            `/users/${userId}`,
            updates
        );

        return response.data;
    }

    async deleteUser(userId: number): Promise<void> {
        await this.client.post<ApiResponse<void>>(`/users/${userId}`, {
            method: 'DELETE',
        });
    }
}

export function createUserApi(baseUrl: string = 'https://api.example.com'): UserApi {
    return new UserApi(baseUrl);
}
