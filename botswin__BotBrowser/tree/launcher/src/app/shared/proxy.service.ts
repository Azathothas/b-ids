import { Injectable } from '@angular/core';
import * as Neutralino from '@neutralinojs/lib';
import type { Proxy } from '../data/proxy';
import { getAppDataPath } from '../utils';

const kProxyConfigFileName = 'proxies.json';

@Injectable({ providedIn: 'root' })
export class ProxyService {
    async getConfigPath(): Promise<string> {
        const basePath = await getAppDataPath('');
        return await Neutralino.filesystem.getJoinedPath(basePath, kProxyConfigFileName);
    }

    async getAllProxies(): Promise<Proxy[]> {
        try {
            const configPath = await this.getConfigPath();
            const content = await Neutralino.filesystem.readFile(configPath);
            const proxies: Proxy[] = JSON.parse(content);
            return proxies.sort((a, b) => (b.createdAt ?? 0) - (a.createdAt ?? 0));
        } catch {
            return [];
        }
    }

    async saveAllProxies(proxies: Proxy[]): Promise<void> {
        const configPath = await this.getConfigPath();
        await Neutralino.filesystem.writeFile(configPath, JSON.stringify(proxies, null, 2));
    }

    async addProxy(proxy: Proxy): Promise<void> {
        const proxies = await this.getAllProxies();
        proxy.createdAt = proxy.createdAt ?? Date.now();
        proxies.push(proxy);
        await this.saveAllProxies(proxies);
    }

    async updateProxy(proxy: Proxy): Promise<void> {
        const proxies = await this.getAllProxies();
        const index = proxies.findIndex((p) => p.id === proxy.id);
        if (index !== -1) {
            proxies[index] = proxy;
            await this.saveAllProxies(proxies);
        }
    }

    async deleteProxy(proxyId: string): Promise<void> {
        const proxies = await this.getAllProxies();
        const filtered = proxies.filter((p) => p.id !== proxyId);
        await this.saveAllProxies(filtered);
    }

    async deleteProxies(proxyIds: string[]): Promise<void> {
        const proxies = await this.getAllProxies();
        const filtered = proxies.filter((p) => !proxyIds.includes(p.id));
        await this.saveAllProxies(filtered);
    }
}
