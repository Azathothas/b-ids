import { inject, Injectable } from '@angular/core';
import type { ParsedProxy } from './proxy-parser.service';
import { ShellService } from './shell.service';

export interface ProxyCheckResult {
    ip: string;
    country: string;
    region: string;
    city: string;
    isp: string;
    org: string;
    hosting: boolean;
}

@Injectable({ providedIn: 'root' })
export class ProxyCheckService {
    readonly #shell = inject(ShellService);

    async checkProxy(proxy: ParsedProxy, signal?: AbortSignal): Promise<ProxyCheckResult> {
        const proxyUrl = this.#buildProxyArg(proxy);
        const apiUrl = 'http://ip-api.com/json?fields=query,country,regionName,city,isp,org,hosting';

        // --max-time caps the whole request so a stalled transfer can't hang forever.
        let curlCmd: string;
        if (proxy.type === 'socks5' || proxy.type === 'socks5h') {
            curlCmd = `curl --socks5-hostname "${proxyUrl}" "${apiUrl}" --connect-timeout 10 --max-time 30 -s`;
        } else {
            curlCmd = `curl -x "${proxyUrl}" "${apiUrl}" --connect-timeout 10 --max-time 30 -s`;
        }

        let result: { exitCode: number; stdOut: string; stdErr: string };
        try {
            result = await this.#shell.run(curlCmd, { signal });
        } catch (error) {
            if (error instanceof DOMException && error.name === 'AbortError') throw error;
            throw new Error(`Failed to execute curl: ${error instanceof Error ? error.message : error}`);
        }

        if (result.stdErr?.trim()) {
            throw new Error(`Proxy check failed: ${result.stdErr.trim()}`);
        }

        const stdout = result.stdOut?.trim();
        if (!stdout) {
            throw new Error('Empty response from IP check service');
        }

        let json: Record<string, unknown>;
        try {
            json = JSON.parse(stdout);
        } catch {
            throw new Error('Invalid response from IP check service');
        }

        return {
            ip: String(json['query'] || ''),
            country: String(json['country'] || ''),
            region: String(json['regionName'] || ''),
            city: String(json['city'] || ''),
            isp: String(json['isp'] || ''),
            org: String(json['org'] || ''),
            hosting: Boolean(json['hosting']),
        };
    }

    #buildProxyArg(proxy: ParsedProxy): string {
        let auth = '';
        if (proxy.username) {
            auth = proxy.password
                ? `${encodeURIComponent(proxy.username)}:${encodeURIComponent(proxy.password)}@`
                : `${encodeURIComponent(proxy.username)}@`;
        }
        // For socks5/socks5h curl expects host:port format (no scheme prefix)
        if (proxy.type === 'socks5' || proxy.type === 'socks5h') {
            return `${auth}${proxy.host}:${proxy.port}`;
        }
        return `${proxy.type}://${auth}${proxy.host}:${proxy.port}`;
    }
}
