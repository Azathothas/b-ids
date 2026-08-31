export type ProxyType = 'http' | 'https' | 'socks5' | 'socks5h';

export const ProxyTypes: ProxyType[] = ['http', 'https', 'socks5', 'socks5h'];

export interface Proxy {
    id: string;
    name: string;
    type: ProxyType;
    host: string;
    port: number;
    username?: string;
    password?: string;
    createdAt?: number;
}
