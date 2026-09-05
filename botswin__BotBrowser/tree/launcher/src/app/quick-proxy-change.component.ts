import { CommonModule } from '@angular/common';
import { Component, inject, type OnInit, ChangeDetectionStrategy } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MAT_DIALOG_DATA, MatDialog, MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { v4 as uuidv4 } from 'uuid';
import type { BrowserProfile } from './data/browser-profile';
import type { Proxy } from './data/proxy';
import { AlertDialogComponent } from './shared/alert-dialog.component';
import { BrowserProfileService } from './shared/browser-profile.service';
import { ProxyInputComponent } from './shared/proxy-input.component';
import { ProxyParserService, type ParsedProxy } from './shared/proxy-parser.service';
import { ProxyService } from './shared/proxy.service';

@Component({
    selector: 'app-quick-proxy-change',
    standalone: true,
    imports: [CommonModule, MatDialogModule, MatButtonModule, MatFormFieldModule, MatSelectModule, ProxyInputComponent],
    template: `
        <h2 mat-dialog-title>Change Proxy - {{ profileName }}</h2>
        <mat-dialog-content>
            @if (proxies.length > 0) {
                <mat-form-field>
                    <mat-label>Select existing proxy</mat-label>
                    <mat-select [(value)]="selectedProxyId" (selectionChange)="onProxySelected($event.value)">
                        <mat-option value="">-- None --</mat-option>
                        @for (proxy of proxies; track proxy.id) {
                            <mat-option [value]="proxy.id">
                                {{ proxy.name }} ({{ proxy.type }}://{{ proxy.host }}:{{ proxy.port }})
                            </mat-option>
                        }
                    </mat-select>
                </mat-form-field>
            }

            <app-proxy-input
                [value]="proxyValue"
                [showQuickParse]="true"
                [showCheckButton]="true"
                [showSaveButton]="!selectedProxyId"
                [showClearButton]="true"
                (valueChange)="onProxyValueChange($event)"
                (saveToList)="onSaveProxyToList($event)"
                (clearProxy)="onClearProxy()"
            />
        </mat-dialog-content>
        <mat-dialog-actions align="end">
            <button mat-button mat-dialog-close>Cancel</button>
            <button mat-button (click)="onSave()">Save</button>
        </mat-dialog-actions>
    `,
    changeDetection: ChangeDetectionStrategy.Eager,
    styles: `
        mat-form-field {
            width: 100%;
        }
    `,
})
export class QuickProxyChangeComponent implements OnInit {
    readonly #proxyService = inject(ProxyService);
    readonly #proxyParser = inject(ProxyParserService);
    readonly #browserProfileService = inject(BrowserProfileService);
    readonly #dialog = inject(MatDialog);
    readonly #dialogRef = inject(MatDialogRef<QuickProxyChangeComponent>);
    readonly #data = inject<BrowserProfile>(MAT_DIALOG_DATA);

    profileName = this.#data.basicInfo.profileName || '';
    proxyValue: ParsedProxy | null = this.#data.proxyServer ? this.#proxyParser.parse(this.#data.proxyServer) : null;
    selectedProxyId = '';
    proxies: Proxy[] = [];

    async ngOnInit() {
        this.proxies = await this.#proxyService.getAllProxies();
    }

    onProxySelected(proxyId: string): void {
        if (!proxyId) return;
        const proxy = this.proxies.find((p) => p.id === proxyId);
        if (proxy) {
            this.proxyValue = {
                type: proxy.type,
                host: proxy.host,
                port: proxy.port,
                username: proxy.username,
                password: proxy.password,
            };
        }
    }

    onProxyValueChange(value: ParsedProxy | null): void {
        this.proxyValue = value;
        this.selectedProxyId = '';
    }

    onClearProxy(): void {
        this.proxyValue = null;
        this.selectedProxyId = '';
    }

    async onSaveProxyToList(proxy: ParsedProxy): Promise<void> {
        const duplicate = this.proxies.find(
            (p) =>
                p.host === proxy.host &&
                p.port === proxy.port &&
                (p.username || '') === (proxy.username || '') &&
                (p.password || '') === (proxy.password || '')
        );
        if (duplicate) {
            this.#dialog.open(AlertDialogComponent, {
                data: { message: `Proxy ${proxy.host}:${proxy.port} already exists in the proxy list.` },
            });
            return;
        }

        await this.#proxyService.addProxy({
            id: uuidv4(),
            name: `${proxy.host}:${proxy.port}`,
            type: proxy.type,
            host: proxy.host,
            port: proxy.port,
            username: proxy.username,
            password: proxy.password,
        });
        this.proxies = await this.#proxyService.getAllProxies();
        this.#dialog.open(AlertDialogComponent, {
            data: { message: `Proxy ${proxy.host}:${proxy.port} saved to proxy list.` },
        });
    }

    async onSave(): Promise<void> {
        const updatedProfile: BrowserProfile = {
            ...this.#data,
            proxyServer: this.proxyValue ? this.#proxyParser.toUrl(this.proxyValue) : undefined,
            updatedAt: Date.now(),
        };

        await this.#browserProfileService.saveBrowserProfile(updatedProfile);
        this.#dialogRef.close(true);
    }
}
