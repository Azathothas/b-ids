import { Injectable } from '@angular/core';
import type { ProxyType } from '../data/proxy';

export interface ParsedProxy {
    type: ProxyType;
    host: string;
    port: number;
    username?: string;
    password?: string;
}

@Injectable({
    providedIn: 'root',
})
export class ProxyParserService {
    parse(input: string): ParsedProxy | null {
        if (!input || !input.trim()) {
            return null;
        }

        input = input.trim();

        // Try standard URL format first
        const urlResult = this.#tryParseUrl(input);
        if (urlResult) {
            return urlResult;
        }

        // Try custom formats
        return this.#tryParseCustomFormat(input);
    }

    #tryParseUrl(input: string): ParsedProxy | null {
        // Add scheme if missing for URL parsing
        let urlString = input;
        if (!input.includes('://')) {
            urlString = 'http://' + input;
        }

        try {
            const url = new URL(urlString);
            const port = parseInt(url.port, 10);
            if (!url.hostname || isNaN(port)) {
                return null;
            }

            const type = this.#parseScheme(url.protocol.replace(':', ''));

            return {
                type,
                host: url.hostname,
                port,
                username: url.username ? decodeURIComponent(url.username) : undefined,
                password: url.password ? decodeURIComponent(url.password) : undefined,
            };
        } catch {
            return null;
        }
    }

    #tryParseCustomFormat(input: string): ParsedProxy | null {
        // Remove scheme if present
        let cleanInput = input;
        let type: ProxyType = 'http';

        const schemeMatch = input.match(/^(https?|socks5h?):\/\//i);
        if (schemeMatch && schemeMatch[1]) {
            type = this.#parseScheme(schemeMatch[1]);
            cleanInput = input.slice(schemeMatch[0].length);
        }

        // Split by @ first
        const atParts = cleanInput.split('@');

        if (atParts.length === 2) {
            // Could be user:pwd@host:port or host:port@user:pwd
            const firstPart = atParts[0] ?? '';
            const secondPart = atParts[1] ?? '';

            const firstColonParts = firstPart.split(':');
            const secondColonParts = secondPart.split(':');

            // Check which part contains host:port (port is pure number)
            if (this.#looksLikeHostPort(firstColonParts)) {
                // host:port@user:pwd
                return this.#buildProxy(
                    type,
                    firstColonParts[0] ?? '',
                    firstColonParts[1] ?? '',
                    secondColonParts[0],
                    secondColonParts[1]
                );
            } else if (this.#looksLikeHostPort(secondColonParts)) {
                // user:pwd@host:port
                return this.#buildProxy(
                    type,
                    secondColonParts[0] ?? '',
                    secondColonParts[1] ?? '',
                    firstColonParts[0],
                    firstColonParts[1]
                );
            }
        }

        // No @ separator, try colon-separated formats
        // host:port:user:pwd or user:pwd:host:port
        const parts = cleanInput.split(':');

        if (parts.length === 2) {
            // host:port (no auth)
            const host = parts[0] ?? '';
            const portStr = parts[1] ?? '';
            const port = parseInt(portStr, 10);
            if (this.#isValidHost(host) && !isNaN(port)) {
                return { type, host, port };
            }
        }

        if (parts.length === 4) {
            // Could be host:port:user:pwd or user:pwd:host:port
            const port1 = parseInt(parts[1] ?? '', 10);
            const port3 = parseInt(parts[3] ?? '', 10);

            if (!isNaN(port1) && this.#isValidHost(parts[0] ?? '')) {
                // host:port:user:pwd
                return this.#buildProxy(type, parts[0] ?? '', parts[1] ?? '', parts[2], parts[3]);
            } else if (!isNaN(port3) && this.#isValidHost(parts[2] ?? '')) {
                // user:pwd:host:port
                return this.#buildProxy(type, parts[2] ?? '', parts[3] ?? '', parts[0], parts[1]);
            }
        }

        return null;
    }

    #parseScheme(scheme: string): ProxyType {
        const lower = scheme.toLowerCase();
        if (lower === 'https') return 'https';
        if (lower === 'socks5h') return 'socks5h';
        if (lower === 'socks5' || lower === 'socks') return 'socks5';
        return 'http';
    }

    #looksLikeHostPort(parts: string[]): boolean {
        if (parts.length !== 2) return false;
        const host = parts[0] ?? '';
        const portStr = parts[1] ?? '';
        const port = parseInt(portStr, 10);
        return this.#isValidHost(host) && !isNaN(port) && port > 0 && port <= 65535;
    }

    #isValidHost(value: string): boolean {
        if (!value) return false;
        // IP address pattern
        const ipPattern = /^(\d{1,3}\.){3}\d{1,3}$/;
        if (ipPattern.test(value)) {
            return true;
        }
        // Domain pattern (contains dot and letters)
        const domainPattern = /^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)*$/;
        return domainPattern.test(value);
    }

    #buildProxy(
        type: ProxyType,
        host: string,
        portStr: string,
        username?: string,
        password?: string
    ): ParsedProxy | null {
        const port = parseInt(portStr, 10);
        if (!this.#isValidHost(host) || isNaN(port) || port <= 0 || port > 65535) {
            return null;
        }

        return {
            type,
            host,
            port,
            username: username || undefined,
            password: password || undefined,
        };
    }

    toUrl(proxy: ParsedProxy): string {
        let url = `${proxy.type}://`;
        if (proxy.username && proxy.password) {
            url += `${encodeURIComponent(proxy.username)}:${encodeURIComponent(proxy.password)}@`;
        } else if (proxy.username) {
            url += `${encodeURIComponent(proxy.username)}@`;
        }
        url += `${proxy.host}:${proxy.port}`;
        return url;
    }
}
