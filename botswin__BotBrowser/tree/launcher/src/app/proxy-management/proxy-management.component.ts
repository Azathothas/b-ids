import { SelectionModel } from '@angular/cdk/collections';
import { CommonModule } from '@angular/common';
import { Component, inject, ViewChild, type AfterViewInit, ChangeDetectionStrategy } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatDialog, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import * as Neutralino from '@neutralinojs/lib';
import type { Proxy } from '../data/proxy';
import { AlertDialogComponent } from '../shared/alert-dialog.component';
import { ConfirmDialogComponent } from '../shared/confirm-dialog.component';
import { ProxyCheckService, type ProxyCheckResult } from '../shared/proxy-check.service';
import { ProxyParserService } from '../shared/proxy-parser.service';
import { ProxyService } from '../shared/proxy.service';
import { StopPropagationDirective } from '../shared/stop-propagation.directive';
import { BulkImportProxyComponent } from './bulk-import-proxy.component';
import { EditProxyComponent } from './edit-proxy.component';

@Component({
    selector: 'app-proxy-management',
    standalone: true,
    imports: [
        CommonModule,
        MatButtonModule,
        MatMenuModule,
        MatDialogModule,
        MatTableModule,
        MatSortModule,
        MatCheckboxModule,
        MatToolbarModule,
        MatIconModule,
        MatProgressBarModule,
        MatProgressSpinnerModule,
        MatTooltipModule,
        MatSnackBarModule,
        StopPropagationDirective,
    ],
    templateUrl: './proxy-management.component.html',
    changeDetection: ChangeDetectionStrategy.Eager,
    styleUrl: './proxy-management.component.scss',
})
export class ProxyManagementComponent implements AfterViewInit {
    readonly #proxyService = inject(ProxyService);
    readonly #proxyCheck = inject(ProxyCheckService);
    readonly #proxyParser = inject(ProxyParserService);
    readonly #dialog = inject(MatDialog);
    readonly #snackBar = inject(MatSnackBar);

    readonly displayedColumns = ['select', 'name', 'type', 'host', 'username', 'status'];
    readonly dataSource = new MatTableDataSource<Proxy>([]);
    readonly selection = new SelectionModel<Proxy>(true, []);
    readonly checkResults = new Map<
        string,
        { status: 'checking' | 'ok' | 'fail'; result?: ProxyCheckResult; error?: string }
    >();

    highlightedId: string | null = null;
    loading = false;
    checking = false;

    @ViewChild(MatSort) set sort(sort: MatSort) {
        if (sort) {
            this.dataSource.sort = sort;
        }
    }

    constructor() {
        this.dataSource.sortingDataAccessor = (data, sortHeaderId) => {
            switch (sortHeaderId) {
                case 'name':
                    return data.name ?? '';
                case 'type':
                    return data.type ?? '';
                case 'host':
                    return data.host ?? '';
                case 'username':
                    return data.username ?? '';
                default:
                    return '';
            }
        };
    }

    async ngAfterViewInit(): Promise<void> {
        await this.refreshProxies();
    }

    formatAddress(proxy: Proxy): string {
        return `${proxy.host}:${proxy.port}`;
    }

    async refreshProxies(): Promise<void> {
        this.loading = true;
        try {
            const proxies = await this.#proxyService.getAllProxies();
            // Preserve checkbox checked state across refresh
            const checkedIds = this.selection.selected.map((p) => p.id);
            this.dataSource.data = proxies;
            this.selection.clear();
            this.selection.select(...proxies.filter((p) => checkedIds.includes(p.id)));
        } finally {
            this.loading = false;
        }
    }

    newProxy(): void {
        this.#dialog
            .open(EditProxyComponent)
            .afterClosed()
            .subscribe((result) => {
                if (!result) return;
                this.highlightedId = result;
                this.refreshProxies().catch(console.error);
            });
    }

    bulkImportProxies(): void {
        this.#dialog
            .open(BulkImportProxyComponent)
            .afterClosed()
            .subscribe(() => {
                this.refreshProxies().catch(console.error);
            });
    }

    editProxy(proxy: Proxy): void {
        this.highlightedId = proxy.id;
        this.#dialog
            .open(EditProxyComponent, { data: proxy })
            .afterClosed()
            .subscribe((result) => {
                if (result) {
                    this.highlightedId = proxy.id;
                }
                this.refreshProxies().catch(console.error);
            });
    }

    editSelectedProxy(): void {
        const proxy = this.highlightedId ? this.dataSource.data.find((p) => p.id === this.highlightedId) : null;
        if (!proxy) return;
        this.editProxy(proxy);
    }

    deleteProxy(proxy: Proxy): void {
        this.#dialog
            .open(ConfirmDialogComponent, {
                data: { message: `Are you sure you want to delete proxy "${proxy.name}"?` },
            })
            .afterClosed()
            .subscribe(async (result: boolean) => {
                if (!result) return;
                await this.#proxyService.deleteProxy(proxy.id);
                await this.refreshProxies();
            });
    }

    deleteProxies(): void {
        if (this.selection.selected.length === 0) return;
        this.#dialog
            .open(ConfirmDialogComponent, {
                data: { message: 'Are you sure you want to delete the selected proxies?' },
            })
            .afterClosed()
            .subscribe(async (result: boolean) => {
                if (!result) return;
                await this.#proxyService.deleteProxies(this.selection.selected.map((p) => p.id));
                await this.refreshProxies();
            });
    }

    async exportProxies(): Promise<void> {
        const targets = this.selection.selected.length > 0 ? this.selection.selected : this.dataSource.data;
        if (targets.length === 0) return;
        const content = targets.map((p) => this.#proxyParser.toUrl(p)).join('\n') + '\n';
        let filePath: string;
        try {
            filePath = await Neutralino.os.showSaveDialog('Export proxies', {
                filters: [{ name: 'Text', extensions: ['txt'] }],
                defaultPath: 'proxies.txt',
            });
        } catch {
            return;
        }
        if (!filePath) return;
        try {
            await Neutralino.filesystem.writeFile(filePath, content);
            this.#snackBar.open(`Exported ${targets.length} proxies`, '', { duration: 2500 });
        } catch (error) {
            this.#dialog.open(AlertDialogComponent, {
                data: { message: `Failed to write file: ${error instanceof Error ? error.message : error}` },
            });
        }
    }

    async checkProxies(): Promise<void> {
        if (this.selection.selected.length === 0) return;
        await this.#checkTargets(this.selection.selected);
    }

    async checkProxy(proxy: Proxy): Promise<void> {
        await this.#checkTargets([proxy]);
    }

    async #checkTargets(targets: Proxy[]): Promise<void> {
        this.checking = true;
        for (const proxy of targets) {
            this.checkResults.set(proxy.id, { status: 'checking' });
        }

        await Promise.all(
            targets.map(async (proxy) => {
                try {
                    const result = await this.#proxyCheck.checkProxy({
                        type: proxy.type,
                        host: proxy.host,
                        port: proxy.port,
                        username: proxy.username,
                        password: proxy.password,
                    });
                    this.checkResults.set(proxy.id, { status: 'ok', result });
                } catch (error) {
                    this.checkResults.set(proxy.id, {
                        status: 'fail',
                        error: error instanceof Error ? error.message : 'Check failed',
                    });
                }
            })
        );
        this.checking = false;
    }

    selectRow(proxy: Proxy): void {
        this.highlightedId = proxy.id;
    }

    get isAllSelected(): boolean {
        return this.selection.selected.length === this.dataSource.data.length;
    }

    toggleAllRows(): void {
        if (this.isAllSelected) {
            this.selection.clear();
        } else {
            this.selection.select(...this.dataSource.data);
        }
    }

    checkboxLabel(row?: Proxy): string {
        if (!row) {
            return `${this.isAllSelected ? 'deselect' : 'select'} all`;
        }
        return `${this.selection.isSelected(row) ? 'deselect' : 'select'} row`;
    }
}
