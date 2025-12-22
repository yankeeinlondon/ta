export const VERSION = "1.0.0";

export interface User {
  id: number;
  name: string;
}

export enum Role {
  Admin,
  User
}

export type ID = string | number;

export default class Store {}
