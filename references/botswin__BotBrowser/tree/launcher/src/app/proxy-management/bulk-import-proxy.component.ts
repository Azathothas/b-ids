import { CommonModule } from '@angular/common';
import { Component, inject, ChangeDetectionStrategy } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { v4 as uuidv4 } from 'uuid';
import type { Proxy } from '../data/proxy';
import { ProxyParserService } from '../shared/proxy-parser.service';
import { ProxyService } from '../shared/proxy.service';

interface ParseResult {
    line: number;
    input: string;
    proxy?: Proxy;
    error?: string;
}

@Component({
    selector: 'app-bulk-import-proxy',
    standalone: true,
    imports: [CommonModule, MatDialogModule, FormsModule, MatFormFieldModule, MatInputModule, MatButtonModule],
    templateUrl: './bulk-import-proxy.component.html',
    changeDetection: ChangeDetectionStrategy.Eager,
    styleUrl: './bulk-import-proxy.component.scss',
})
export class BulkImportProxyComponent {
    readonly #proxyService = inject(ProxyService);
    readonly #proxyParser = inject(ProxyParserService);
    readonly #dialogRef = inject(MatDialogRef<BulkImportProxyComponent>);

    input = '';
    results: ParseResult[] = [];
    importing = false;

    get validCount(): number {
        return this.results.filter((r) => r.proxy).length;
    }

    get errorCount(): number {
        return this.results.filter((r) => r.error).length;
    }

    onInputChange(): void {
        this.results = [];
        if (!this.input.trim()) {
            return;
        }

        const lines = this.input.split('\n').filter((line) => line.trim());
        this.results = lines.map((line, index) => this.#parseLine(line.trim(), index + 1));
    }

    #parseLine(input: string, lineNumber: number): ParseResult {
        const parsed = this.#proxyParser.parse(input);
        if (!parsed) {
            return { line: lineNumber, input, error: 'Invalid format' };
        }

        const proxy: Proxy = {
            id: uuidv4(),
            name: `${parsed.host}:${parsed.port}`,
            type: parsed.type,
            host: parsed.host,
            port: parsed.port,
            username: parsed.username,
            password: parsed.password,
        };

        return { line: lineNumber, input, proxy };
    }

    async onImportClick(): Promise<void> {
        const validProxies = this.results.filter((r) => r.proxy).map((r) => r.proxy!);
        if (validProxies.length === 0) {
            return;
        }

        this.importing = true;
        try {
            for (const proxy of validProxies) {
                await this.#proxyService.addProxy(proxy);
            }
            this.#dialogRef.close(validProxies.length);
        } finally {
            this.importing = false;
        }
    }
}
