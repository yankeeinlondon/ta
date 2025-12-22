// Type definitions for the project

export interface User {
    id: number;
    name: string;
    email: string;
    role: UserRole;
}

export type UserRole = 'admin' | 'user' | 'guest';

export interface ApiResponse<T> {
    data: T;
    status: number;
    message?: string;
}

export interface NetworkConfig {
    baseUrl: string;
    timeout: number;
    retries: number;
}

export enum HttpMethod {
    GET = 'GET',
    POST = 'POST',
    PUT = 'PUT',
    DELETE = 'DELETE',
}

export type RequestOptions = {
    method: HttpMethod;
    headers?: Record<string, string>;
    body?: any;
    timeout?: number;
};
