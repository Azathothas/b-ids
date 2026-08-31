import { CommonModule } from '@angular/common';
import { Component, effect, inject, type OnInit, ChangeDetectionStrategy } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatTooltipModule } from '@angular/material/tooltip';
import * as Neutralino from '@neutralinojs/lib';
import { getLatestAssetForPlatform, type InstalledKernel, type KernelRelease } from '../data/kernel';
import { AlertDialogComponent } from '../shared/alert-dialog.component';
import { ConfirmDialogComponent } from '../shared/confirm-dialog.component';
import { KernelService } from '../shared/kernel.service';

@Component({
    selector: 'app-kernel-management',
    standalone: true,
    imports: [
        CommonModule,
        MatButtonModule,
        MatDialogModule,
        MatTableModule,
        MatToolbarModule,
        MatIconModule,
        MatProgressBarModule,
        MatTooltipModule,
    ],
    templateUrl: './kernel-management.component.html',
    changeDetection: ChangeDetectionStrategy.Eager,
    styleUrl: './kernel-management.component.scss',
})
export class KernelManagementComponent implements OnInit {
    readonly kernelService = inject(KernelService);
    readonly #dialog = inject(MatDialog);

    readonly displayedColumns = ['version', 'platform', 'installedAt', 'actions'];
    readonly dataSource = new MatTableDataSource<InstalledKernel>([]);

    availableReleases: KernelRelease[] = [];
    loading = false;
    loadingReleases = false;
    error: string | null = null;

    constructor() {
        effect(() => {
            const version = this.kernelService.installedKernelsVersion();
            if (version > 0) this.refresh();
        });
    }

    // Check if a specific release is being downloaded
    isDownloading(tagName: string): boolean {
        return this.kernelService.isDownloading(tagName);
    }

    // Check if there are any active downloads
    get hasActiveDownloads(): boolean {
        return this.kernelService.hasActiveDownloads();
    }

    // Get all download progresses for display
    get downloadProgresses() {
        return this.kernelService.downloadProgresses();
    }

    async ngOnInit(): Promise<void> {
        await this.kernelService.initialize();
        await this.refresh();
        await this.loadAvailableReleases();
    }

    async refresh(): Promise<void> {
        this.loading = true;
        try {
            const kernels = await this.kernelService.getInstalledKernels();
            this.dataSource.data = kernels;
        } finally {
            this.loading = false;
        }
    }

    async loadAvailableReleases(): Promise<void> {
        this.loadingReleases = true;
        this.error = null;
        try {
            this.availableReleases = await this.kernelService.fetchAvailableReleases();
        } catch (e) {
            this.error = e instanceof Error ? e.message : 'Failed to load releases';
        } finally {
            this.loadingReleases = false;
        }
    }

    getAvailableForDownload(): KernelRelease[] {
        // Filter by majorVersion - if a major version is installed, don't show it
        // (auto-update will handle upgrades to newer releases within the same major version)
        const installedMajorVersions = new Set(this.dataSource.data.map((k) => k.majorVersion));
        const platform = this.kernelService.getCurrentPlatform();
        return this.availableReleases.filter(
            (r) => !installedMajorVersions.has(r.majorVersion) && r.assets.some((a) => a.platform === platform)
        );
    }

    async downloadKernel(release: KernelRelease): Promise<void> {
        try {
            await this.kernelService.downloadAndInstall(release);
            await this.refresh();
        } catch (e) {
            this.error = e instanceof Error ? e.message : 'Download failed';
        }
    }

    async openInstallDir(kernel: InstalledKernel): Promise<void> {
        try {
            await Neutralino.os.open(kernel.installPath);
        } catch (error) {
            console.error('Failed to open directory:', error);
        }
    }

    deleteKernel(kernel: InstalledKernel): void {
        this.#dialog
            .open(ConfirmDialogComponent, {
                data: { message: `Are you sure you want to delete kernel ${kernel.fullVersion}?` },
            })
            .afterClosed()
            .subscribe(async (result: boolean) => {
                if (!result) return;
                try {
                    await this.kernelService.deleteKernel(kernel.id);
                    await this.refresh();
                } catch (error) {
                    this.#dialog.open(AlertDialogComponent, {
                        data: { message: error instanceof Error ? error.message : 'Failed to delete kernel.' },
                    });
                }
            });
    }

    formatDate(timestamp: number): string {
        return new Date(timestamp).toLocaleDateString();
    }

    formatAssetDate(assetDate?: string): string | null {
        if (!assetDate || assetDate.length !== 8) return null;
        const year = Number(assetDate.slice(0, 4));
        const month = Number(assetDate.slice(4, 6));
        const day = Number(assetDate.slice(6, 8));
        return new Date(year, month - 1, day).toLocaleDateString();
    }

    getReleaseAssetDate(release: KernelRelease): string | null {
        const platform = this.kernelService.getCurrentPlatform();
        const asset = getLatestAssetForPlatform(release.assets, platform);
        return this.formatAssetDate(asset?.assetDate);
    }

    formatSize(bytes: number): string {
        const mb = bytes / (1024 * 1024);
        return `${mb.toFixed(1)} MB`;
    }

    getPlatformLabel(platform: string): string {
        const labels: Record<string, string> = {
            win_x86_64: 'Windows x64',
            mac_arm64: 'macOS ARM64',
            mac_x86_64: 'macOS Intel',
            linux_x86_64: 'Linux x64',
            linux_arm64: 'Linux ARM64',
        };
        return labels[platform] || platform;
    }
}
