import { inject, Injectable, signal } from '@angular/core';
import * as Neutralino from '@neutralinojs/lib';
import { ShellService } from './shell.service';
import { v4 as uuidv4 } from 'uuid';
import { AppName } from '../const';
import {
    compareVersions,
    getAssetDate,
    getCurrentPlatform,
    getLatestAssetForPlatform,
    getPlatformFromAssetName,
    parseVersion,
    type DownloadProgress,
    type InstalledKernel,
    type KernelAsset,
    type KernelPlatform,
    type KernelRelease,
} from '../data/kernel';

const GITHUB_API_URL = 'https://api.github.com/repos/botswin/BotBrowser/releases';
const KERNELS_FILE = 'kernels.json';

interface GitHubRelease {
    tag_name: string;
    name: string;
    published_at: string;
    assets: {
        id: number;
        name: string;
        browser_download_url: string;
        size: number;
    }[];
}

@Injectable({ providedIn: 'root' })
export class KernelService {
    readonly #shell = inject(ShellService);

    // Support multiple concurrent downloads - use tagName as key
    readonly downloadProgresses = signal<DownloadProgress[]>([]);
    readonly isAutoUpdating = signal(false);
    readonly installedKernelsVersion = signal(0);

    #installedKernels: InstalledKernel[] = [];
    #currentPlatform: KernelPlatform = 'win_x86_64';
    #allReleases: KernelRelease[] = [];
    #initialized = false;
    #extractorAvailable: boolean | null = null; // null = not checked yet
    #intervalId: ReturnType<typeof setInterval> | null = null;

    async initialize(): Promise<void> {
        if (this.#initialized) return;

        const osInfo = await Neutralino.computer.getOSInfo();
        const arch = await this.#getArchitecture();
        this.#currentPlatform = getCurrentPlatform(osInfo.name, arch);
        await this.#loadInstalledKernels();
        await this.#checkExtractorAvailable();
        this.#initialized = true;
    }

    isExtractorAvailable(): boolean {
        return this.#extractorAvailable === true;
    }

    isExtractorChecked(): boolean {
        return this.#extractorAvailable !== null;
    }

    async #checkExtractorAvailable(): Promise<void> {
        if (this.#extractorAvailable !== null) return;

        if (this.#currentPlatform === 'win_x86_64') {
            // Check 7z on Windows
            const paths = [
                '7z',
                'C:\\Program Files\\7-Zip\\7z.exe',
                'C:\\Program Files (x86)\\7-Zip\\7z.exe',
            ];
            // Also check registry
            try {
                const reg = await this.#shell.run(
                    'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\7-Zip" /v Path 2>nul'
                );
                const match = reg.stdOut.match(/Path\s+REG_SZ\s+(.+)/);
                if (match?.[1]) paths.unshift(`${match[1].trim()}\\7z.exe`);
            } catch { /* ignore */ }
            try {
                const reg = await this.#shell.run(
                    'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\7-Zip" /v Path 2>nul'
                );
                const match = reg.stdOut.match(/Path\s+REG_SZ\s+(.+)/);
                if (match?.[1]) paths.unshift(`${match[1].trim()}\\7z.exe`);
            } catch { /* ignore */ }

            for (const p of paths) {
                try {
                    const r = await this.#shell.run(`"${p}" i 2>nul`);
                    if (r.stdOut.includes('7-Zip')) {
                        this.#extractorAvailable = true;
                        return;
                    }
                } catch { /* try next */ }
            }
            this.#extractorAvailable = false;
        } else if (this.#currentPlatform.startsWith('mac')) {
            // macOS: hdiutil is always available (system tool)
            this.#extractorAvailable = true;
        } else {
            // Linux: need dpkg-deb or (ar + tar) for .deb extraction
            try {
                const r = await this.#shell.run('command -v dpkg-deb 2>/dev/null');
                if (r.stdOut.trim()) {
                    this.#extractorAvailable = true;
                    return;
                }
            } catch { /* not found */ }
            try {
                const rAr = await this.#shell.run('command -v ar 2>/dev/null');
                const rTar = await this.#shell.run('command -v tar 2>/dev/null');
                if (rAr.stdOut.trim() && rTar.stdOut.trim()) {
                    this.#extractorAvailable = true;
                    return;
                }
            } catch { /* not found */ }
            this.#extractorAvailable = false;
        }
    }

    /** Run the kernel auto-update routine now, then re-check periodically (default: hourly). */
    startPeriodicCheck(intervalMs = 60 * 60 * 1000): void {
        if (this.#intervalId) return;
        this.performStartupTasks().catch(console.error);
        this.#intervalId = setInterval(() => {
            // Skip if a previous run or any download (auto or manual) is still in progress
            if (this.isAutoUpdating() || this.hasActiveDownloads()) return;
            this.performStartupTasks().catch(console.error);
        }, intervalMs);
    }

    // Fetch releases and auto-update/-download kernels.
    // Called on startup and on each periodic check; re-entrant calls are ignored.
    async performStartupTasks(): Promise<void> {
        if (this.isAutoUpdating()) return; // prevent overlapping runs

        try {
            this.isAutoUpdating.set(true);
            await this.initialize();

            // Always re-fetch so newly published releases are detected on periodic runs
            this.#allReleases = await this.#fetchAllReleases();

            // Clean up old kernel versions (keep only the latest per major version)
            await this.#cleanupOldKernels();

            // Auto-update installed kernels to newer releases
            await this.#autoUpdateKernels();

            // Clean up again after auto-update to remove old versions replaced by new downloads
            await this.#cleanupOldKernels();

            // Auto-download the latest release if no kernels installed
            await this.#autoDownloadLatest();
        } catch (error) {
            console.error('Startup tasks failed:', error);
        } finally {
            this.isAutoUpdating.set(false);
        }
    }

    async #getArchitecture(): Promise<string> {
        try {
            const osInfo = await Neutralino.computer.getOSInfo();
            if (osInfo.name.includes('Windows')) {
                const result = await this.#shell.run('echo %PROCESSOR_ARCHITECTURE%');
                return result.stdOut.trim();
            } else {
                const result = await this.#shell.run('uname -m');
                return result.stdOut.trim();
            }
        } catch {
            return 'x86_64';
        }
    }

    getCurrentPlatform(): KernelPlatform {
        return this.#currentPlatform;
    }

    // Fetch all releases from GitHub (raw, unfiltered)
    async #fetchAllReleases(): Promise<KernelRelease[]> {
        const response = await fetch(GITHUB_API_URL);
        if (!response.ok) {
            throw new Error(`Failed to fetch releases: ${response.statusText}`);
        }

        const releases: GitHubRelease[] = await response.json();
        const kernelReleases: KernelRelease[] = [];

        for (const release of releases) {
            const assets: KernelAsset[] = [];

            for (const asset of release.assets) {
                const platform = getPlatformFromAssetName(asset.name);
                if (platform) {
                    assets.push({
                        id: asset.id,
                        name: asset.name,
                        platform,
                        downloadUrl: asset.browser_download_url,
                        size: asset.size,
                        assetDate: getAssetDate(asset.name),
                    });
                }
            }

            // Only include releases with binary assets
            if (assets.length > 0) {
                const { major, full, isValidMajor } = parseVersion(release.tag_name);

                // Filter out releases without valid major version (e.g., "20250728")
                if (isValidMajor) {
                    kernelReleases.push({
                        tagName: release.tag_name,
                        name: release.name,
                        majorVersion: major,
                        fullVersion: full,
                        publishedAt: release.published_at,
                        assets,
                    });
                }
            }
        }

        return kernelReleases;
    }

    // Get available releases (only latest per major version, for current platform)
    async fetchAvailableReleases(): Promise<KernelRelease[]> {
        if (this.#allReleases.length === 0) {
            this.#allReleases = await this.#fetchAllReleases();
        }

        // Filter to only releases that have assets for current platform
        const platformReleases = this.#allReleases.filter((r) =>
            r.assets.some((a) => a.platform === this.#currentPlatform)
        );

        // Group by major version and keep only the latest (highest full version) for each
        const latestByMajor = new Map<number, KernelRelease>();

        for (const release of platformReleases) {
            const existing = latestByMajor.get(release.majorVersion);
            if (!existing || compareVersions(release.fullVersion, existing.fullVersion) > 0) {
                latestByMajor.set(release.majorVersion, release);
            }
        }

        // Sort by major version descending
        return Array.from(latestByMajor.values()).sort((a, b) => b.majorVersion - a.majorVersion);
    }

    async getInstalledKernels(): Promise<InstalledKernel[]> {
        return this.#installedKernels;
    }

    async getInstalledKernelById(id: string): Promise<InstalledKernel | undefined> {
        return this.#installedKernels.find((k) => k.id === id);
    }

    // Highest available major version on GitHub for the current platform (WebKit zero-kernel auto-download).
    async getLatestAvailableMajorVersion(): Promise<number | null> {
        if (this.#allReleases.length === 0) {
            try { this.#allReleases = await this.#fetchAllReleases(); }
            catch { return null; }
        }
        const candidates = this.#allReleases
            .filter((r) => r.assets.some((a) => a.platform === this.#currentPlatform))
            .map((r) => r.majorVersion);
        return candidates.length > 0 ? Math.max(...candidates) : null;
    }

    // Latest installed kernel across all major versions; used as WebKit/Safari fallback.
    async getLatestInstalledKernel(): Promise<InstalledKernel | undefined> {
        const platformKernels = this.#installedKernels.filter((k) => k.platform === this.#currentPlatform);
        if (platformKernels.length === 0) return undefined;
        return platformKernels.reduce((latest, k) => {
            if (k.majorVersion !== latest.majorVersion) {
                return k.majorVersion > latest.majorVersion ? k : latest;
            }
            const versionCompare = compareVersions(k.fullVersion, latest.fullVersion);
            if (versionCompare !== 0) return versionCompare > 0 ? k : latest;
            const dateK = k.assetDate || '';
            const dateLatest = latest.assetDate || '';
            return dateK > dateLatest ? k : latest;
        });
    }

    async getInstalledKernelByMajorVersion(majorVersion: number): Promise<InstalledKernel | undefined> {
        // Get the latest installed kernel for this major version
        const kernels = this.#installedKernels.filter(
            (k) => k.majorVersion === majorVersion && k.platform === this.#currentPlatform
        );

        if (kernels.length === 0) return undefined;

        // Return the one with highest version, then highest assetDate
        return kernels.reduce((latest, k) => {
            const versionCompare = compareVersions(k.fullVersion, latest.fullVersion);
            if (versionCompare > 0) return k;
            if (versionCompare < 0) return latest;
            // If versions are equal, compare by assetDate
            const dateK = k.assetDate || '';
            const dateLatest = latest.assetDate || '';
            return dateK > dateLatest ? k : latest;
        });
    }

    // Find a release by major version and start downloading it
    async downloadKernelByMajorVersion(majorVersion: number): Promise<void> {
        // Fetch releases if not already cached
        if (this.#allReleases.length === 0) {
            this.#allReleases = await this.#fetchAllReleases();
        }

        // Find releases for this major version
        const releases = this.#allReleases.filter(
            (r) => r.majorVersion === majorVersion && r.assets.some((a) => a.platform === this.#currentPlatform)
        );

        if (releases.length === 0) {
            throw new Error(`No kernel available for version ${majorVersion}`);
        }

        // Sort by version descending to get the latest
        releases.sort((a, b) => compareVersions(b.fullVersion, a.fullVersion));
        const latestRelease = releases[0]!;

        // Check if already downloading
        if (this.isDownloading(latestRelease.tagName)) {
            return; // Already downloading, no need to start again
        }

        // Start download (don't await - let it run in background)
        this.downloadAndInstall(latestRelease).catch((error) => {
            console.error(`Failed to download kernel ${majorVersion}:`, error);
        });
    }

    /**
     * Called when a browser using this kernel exits.
     * If this kernel is not the latest for its major version, delete it.
     */
    async cleanupOldKernelIfOutdated(kernelId: string): Promise<void> {
        const kernel = this.#installedKernels.find((k) => k.id === kernelId);
        if (!kernel) return;

        const latest = await this.getInstalledKernelByMajorVersion(kernel.majorVersion);
        if (!latest || latest.id === kernel.id) return; // this IS the latest, nothing to clean

        console.log(`Runtime cleanup: removing outdated kernel ${kernel.fullVersion} (${kernel.assetDate})`);
        const deleted = await this.#deleteDirectory(kernel.installPath);
        if (deleted) {
            this.#installedKernels = this.#installedKernels.filter((k) => k.id !== kernel.id);
            await this.#saveInstalledKernels();
            this.installedKernelsVersion.update((v) => v + 1);
        }
    }

    // Clean up old kernel versions, keeping only the latest per major version
    async #cleanupOldKernels(): Promise<void> {
        const kernelsByMajor = new Map<number, InstalledKernel[]>();

        // Group kernels by major version
        for (const kernel of this.#installedKernels) {
            if (kernel.platform !== this.#currentPlatform) continue;

            const list = kernelsByMajor.get(kernel.majorVersion) || [];
            list.push(kernel);
            kernelsByMajor.set(kernel.majorVersion, list);
        }

        // For each major version with multiple kernels, delete older ones
        for (const [majorVersion, kernels] of kernelsByMajor) {
            if (kernels.length <= 1) continue;

            // Sort by version descending, then by assetDate descending
            kernels.sort((a, b) => {
                const versionCompare = compareVersions(b.fullVersion, a.fullVersion);
                if (versionCompare !== 0) return versionCompare;
                // If versions are equal, compare by assetDate
                const dateA = a.assetDate || '';
                const dateB = b.assetDate || '';
                return dateB.localeCompare(dateA);
            });

            // Keep the first (latest), delete the rest
            const toDelete = kernels.slice(1);
            for (const kernel of toDelete) {
                console.log(
                    `Cleaning up old kernel: ${kernel.fullVersion} (${kernel.assetDate}) (major ${majorVersion})`
                );
                try {
                    // Safety: only delete directory if no other kernel shares it
                    const normPath = (p: string) => p.replace(/[\\/]+$/, '').toLowerCase();
                    const pathShared = this.#installedKernels.some(
                        (k) => k.id !== kernel.id && normPath(k.installPath) === normPath(kernel.installPath)
                    );
                    let deleted = true;
                    if (!pathShared) {
                        deleted = await this.#deleteDirectory(kernel.installPath);
                    }
                    if (deleted || pathShared) {
                        // Only remove record if directory is gone or shared with another kernel
                        this.#installedKernels = this.#installedKernels.filter((k) => k.id !== kernel.id);
                    } else {
                        console.log(`Directory still locked, keeping kernel record: ${kernel.installPath}`);
                    }
                } catch (error) {
                    console.error(`Failed to delete old kernel ${kernel.fullVersion}:`, error);
                }
            }
        }

        await this.#saveInstalledKernels();
    }

    // Auto-update installed kernels to newer releases
    async #autoUpdateKernels(): Promise<void> {
        const latestReleases = await this.fetchAvailableReleases();

        for (const release of latestReleases) {
            const installedKernel = await this.getInstalledKernelByMajorVersion(release.majorVersion);

            if (installedKernel) {
                // Get the latest asset for current platform (highest assetDate within the release)
                const latestAsset = getLatestAssetForPlatform(release.assets, this.#currentPlatform);
                if (!latestAsset) continue;

                // Check if there's a newer version OR newer asset date
                const versionCompare = compareVersions(release.fullVersion, installedKernel.fullVersion);
                const installedAssetDate = installedKernel.assetDate || '';
                const latestAssetDate = latestAsset.assetDate || '';
                const hasNewerAssetDate = latestAssetDate > installedAssetDate;

                if (versionCompare > 0 || (versionCompare === 0 && hasNewerAssetDate)) {
                    console.log(
                        `Auto-updating kernel ${release.majorVersion}: ${installedKernel.fullVersion} (${installedAssetDate}) -> ${release.fullVersion} (${latestAssetDate})`
                    );
                    try {
                        await this.downloadAndInstall(release);
                    } catch (error) {
                        console.error(`Failed to auto-update kernel ${release.majorVersion}:`, error);
                    }
                }
            }
        }
    }

    // Auto-download the latest release if no kernels installed
    async #autoDownloadLatest(): Promise<void> {
        // Check if we have any kernels for current platform
        const platformKernels = this.#installedKernels.filter((k) => k.platform === this.#currentPlatform);

        if (platformKernels.length > 0) {
            return; // Already have kernels, no need to auto-download
        }

        const latestReleases = await this.fetchAvailableReleases();
        if (latestReleases.length === 0) {
            console.log('No releases available for auto-download');
            return;
        }

        // Download the latest release (first one, as they're sorted by major version descending)
        const latestRelease = latestReleases[0];
        if (!latestRelease) {
            return;
        }
        console.log(`Auto-downloading latest kernel: ${latestRelease.fullVersion}`);

        try {
            await this.downloadAndInstall(latestRelease);
        } catch (error) {
            console.error('Failed to auto-download latest kernel:', error);
        }
    }

    // Check if a specific release is currently being downloaded
    isDownloading(tagName: string): boolean {
        return this.downloadProgresses().some(
            (p) => p.tagName === tagName && (p.status === 'downloading' || p.status === 'extracting')
        );
    }

    // Check if any download is in progress
    hasActiveDownloads(): boolean {
        return this.downloadProgresses().some((p) => p.status === 'downloading' || p.status === 'extracting');
    }

    async downloadAndInstall(release: KernelRelease): Promise<InstalledKernel> {
        // Check extractor availability before attempting download
        await this.#checkExtractorAvailable();
        if (!this.#extractorAvailable) {
            if (this.#currentPlatform === 'win_x86_64') {
                throw new Error('7-Zip is required to extract kernels. Please install it from https://7-zip.org/');
            } else {
                throw new Error('dpkg-deb (or ar + tar) is required to extract kernels. Please install dpkg or binutils.');
            }
        }

        // Check if this specific release is already being downloaded
        if (this.isDownloading(release.tagName)) {
            throw new Error(`${release.tagName} is already being downloaded.`);
        }

        // Pick the latest asset by assetDate so we install the freshest build within the release
        const asset = getLatestAssetForPlatform(release.assets, this.#currentPlatform);
        if (!asset) {
            throw new Error(`No binary available for platform: ${this.#currentPlatform}`);
        }

        const kernelsDir = await this.#getKernelsDirectory();
        const tempDir = await this.#getTempDirectory();
        const downloadPath = await Neutralino.filesystem.getJoinedPath(tempDir, asset.name);

        // Add progress entry for this download
        this.#updateProgress(release.tagName, {
            tagName: release.tagName,
            platform: this.#currentPlatform,
            downloadedBytes: 0,
            totalBytes: asset.size,
            status: 'downloading',
        });

        try {
            // Download the file
            await this.#downloadFile(asset.downloadUrl, downloadPath, asset.size, release.tagName);

            // Extract to a unique directory using GitHub asset ID to avoid collisions
            this.#updateProgress(release.tagName, { status: 'extracting' });

            const installDirName = `${release.tagName}_${asset.id}`;
            const installDir = await Neutralino.filesystem.getJoinedPath(kernelsDir, installDirName);
            await this.#extract(downloadPath, installDir);

            // Clean up download file
            await this.#deleteFile(downloadPath);

            // Find executable path
            const executablePath = await this.#findExecutable(installDir);

            const kernel: InstalledKernel = {
                id: uuidv4(),
                tagName: release.tagName,
                majorVersion: release.majorVersion,
                fullVersion: release.fullVersion,
                platform: this.#currentPlatform,
                installPath: installDir,
                executablePath,
                installedAt: Date.now(),
                assetName: asset.name,
                assetDate: asset.assetDate,
            };

            this.#installedKernels.push(kernel);
            await this.#saveInstalledKernels();

            // Mark as completed and remove after delay
            this.#updateProgress(release.tagName, { status: 'completed' });
            setTimeout(() => this.#removeProgress(release.tagName), 2000);

            return kernel;
        } catch (error) {
            this.#updateProgress(release.tagName, {
                status: 'failed',
                error: error instanceof Error ? error.message : String(error),
            });
            throw error;
        }
    }

    // Update progress for a specific download
    #updateProgress(tagName: string, updates: Partial<DownloadProgress>): void {
        this.downloadProgresses.update((list) => {
            const index = list.findIndex((p) => p.tagName === tagName);
            if (index >= 0) {
                const updated = [...list];
                updated[index] = { ...updated[index]!, ...updates };
                return updated;
            } else {
                return [...list, updates as DownloadProgress];
            }
        });
    }

    // Remove progress entry for a specific download
    #removeProgress(tagName: string): void {
        this.downloadProgresses.update((list) => list.filter((p) => p.tagName !== tagName));
    }

    async deleteKernel(id: string): Promise<void> {
        const kernel = this.#installedKernels.find((k) => k.id === id);
        if (!kernel) return;

        const deleted = await this.#deleteDirectory(kernel.installPath);
        if (!deleted) {
            throw new Error('Cannot delete kernel: files may be in use. Please close all browsers using this kernel first.');
        }

        this.#installedKernels = this.#installedKernels.filter((k) => k.id !== id);
        await this.#saveInstalledKernels();
    }

    async #getKernelsDirectory(): Promise<string> {
        const dataPath = await Neutralino.os.getPath('data');
        const kernelsDir = await Neutralino.filesystem.getJoinedPath(dataPath, AppName, 'kernels');

        try {
            await Neutralino.filesystem.getStats(kernelsDir);
        } catch {
            await Neutralino.filesystem.createDirectory(kernelsDir);
        }

        return kernelsDir;
    }

    async #getTempDirectory(): Promise<string> {
        const tempPath = await Neutralino.os.getPath('temp');
        const appTempDir = await Neutralino.filesystem.getJoinedPath(tempPath, AppName, 'downloads');

        try {
            await Neutralino.filesystem.getStats(appTempDir);
        } catch {
            await Neutralino.filesystem.createDirectory(appTempDir);
        }

        return appTempDir;
    }

    async #deleteFile(filePath: string): Promise<void> {
        try {
            const osInfo = await Neutralino.computer.getOSInfo();
            if (osInfo.name.includes('Windows')) {
                await this.#shell.exec(`del /f "${filePath}"`);
            } else {
                await this.#shell.exec(`rm -f "${filePath}"`);
            }
        } catch {
            // Ignore cleanup errors
        }
    }

    async #deleteDirectory(dirPath: string): Promise<boolean> {
        try {
            const osInfo = await Neutralino.computer.getOSInfo();
            if (osInfo.name.includes('Windows')) {
                await this.#shell.exec(`rmdir /s /q "${dirPath}"`);
            } else {
                await this.#shell.exec(`rm -rf "${dirPath}"`);
            }
            // Verify directory is actually gone
            try {
                await Neutralino.filesystem.getStats(dirPath);
                return false; // still exists (locked files on Windows)
            } catch {
                return true; // gone
            }
        } catch {
            return false;
        }
    }

    async #downloadFile(url: string, destPath: string, totalSize: number, tagName: string): Promise<void> {
        const osInfo = await Neutralino.computer.getOSInfo();

        // Start progress monitoring
        let progressInterval: ReturnType<typeof setInterval> | null = null;
        const startProgressMonitoring = () => {
            progressInterval = setInterval(async () => {
                try {
                    const stats = await Neutralino.filesystem.getStats(destPath);
                    if (stats.size > 0) {
                        this.#updateProgress(tagName, { downloadedBytes: stats.size });
                    }
                } catch {
                    // File doesn't exist yet, ignore
                }
            }, 500);
        };

        const stopProgressMonitoring = () => {
            if (progressInterval) {
                clearInterval(progressInterval);
                progressInterval = null;
            }
        };

        startProgressMonitoring();

        try {
            // Use spawnProcess for non-blocking download
            if (osInfo.name.includes('Windows')) {
                // Use curl if available, fallback to PowerShell
                try {
                    const exitCode = await this.#shell.exec(`curl -L -o "${destPath}" "${url}"`);
                    console.log('curl download completed with exit code:', exitCode);
                    if (exitCode !== 0) {
                        throw new Error(`curl failed with exit code ${exitCode}`);
                    }
                } catch {
                    // Fallback to PowerShell
                    const psScript = `$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '${url}' -OutFile '${destPath}' -UseBasicParsing`;
                    const exitCode = await this.#shell.exec(`powershell -Command "${psScript}"`);
                    if (exitCode !== 0) {
                        throw new Error(`PowerShell download failed with exit code ${exitCode}`);
                    }
                }
            } else {
                // Use curl for macOS/Linux
                const exitCode = await this.#shell.exec(`curl -L -o "${destPath}" "${url}"`);
                if (exitCode !== 0) {
                    throw new Error(`curl failed with exit code ${exitCode}`);
                }
            }
        } finally {
            stopProgressMonitoring();
        }

        // Verify download succeeded
        try {
            const stats = await Neutralino.filesystem.getStats(destPath);
            console.log(`Downloaded file size: ${stats.size} bytes (expected: ${totalSize})`);
            if (stats.size < totalSize * 0.9) {
                throw new Error(`Downloaded file is too small: ${stats.size} bytes (expected ~${totalSize} bytes)`);
            }
        } catch (e) {
            if (e instanceof Error && e.message.includes('too small')) {
                throw e;
            }
            throw new Error(`Download verification failed: file not found at ${destPath}`);
        }

        // Update progress to 100%
        this.#updateProgress(tagName, { downloadedBytes: totalSize });
    }

    async #extract(archivePath: string, destDir: string): Promise<void> {
        const osInfo = await Neutralino.computer.getOSInfo();

        console.log('=== EXTRACT START ===');
        console.log('Archive path:', archivePath);
        console.log('Destination dir:', destDir);
        console.log('OS name:', osInfo.name);

        // Create destination directory
        try {
            await Neutralino.filesystem.getStats(destDir);
        } catch {
            await Neutralino.filesystem.createDirectory(destDir);
        }

        if (osInfo.name.includes('Windows')) {
            // Extract 7z using 7z command
            if (archivePath.endsWith('.7z')) {
                await this.#extract7z(archivePath, destDir);

                // Check for nested .7z files inside (7z may contain another 7z)
                // Use /s for recursive search, /b for bare format
                let nested7zFiles: string[] = [];
                try {
                    const result7z = await this.#shell.run(`dir /s /b "${destDir}\\*.7z" 2>nul`);
                    nested7zFiles = result7z.stdOut
                        .trim()
                        .split('\n')
                        .map((f) => f.trim())
                        .filter((f) => f.endsWith('.7z'));
                } catch { /* no nested 7z files found */ }

                for (const nested7zFullPath of nested7zFiles) {
                    console.log(`Extracting nested 7z: ${nested7zFullPath}`);
                    await this.#extract7z(nested7zFullPath, destDir);
                    await this.#deleteFile(nested7zFullPath);
                }

                // Check for nested .zip files inside (7z may contain a zip)
                let zipFiles: string[] = [];
                try {
                    const result = await this.#shell.run(`dir /s /b "${destDir}\\*.zip" 2>nul`);
                    zipFiles = result.stdOut
                        .trim()
                        .split('\n')
                        .map((f) => f.trim())
                        .filter((f) => f.endsWith('.zip'));
                } catch { /* no nested zip files found */ }

                for (const nestedZipFullPath of zipFiles) {
                    console.log(`Extracting nested zip: ${nestedZipFullPath}`);
                    await this.#shell.exec(
                        `powershell -Command "Expand-Archive -Path '${nestedZipFullPath}' -DestinationPath '${destDir}' -Force"`
                    );
                    await this.#deleteFile(nestedZipFullPath);
                }
            } else if (archivePath.endsWith('.zip')) {
                await this.#shell.run(
                    `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`
                );
            }
        } else if (osInfo.name.includes('Darwin')) {
            // macOS: mount dmg and copy
            if (archivePath.endsWith('.dmg')) {
                const mountPoint = `/Volumes/BotBrowser_${Date.now()}`;
                try {
                    // Mount the dmg (use -quiet to reduce output, -noverify to skip verification for speed)
                    await this.#shell.run(
                        `hdiutil attach "${archivePath}" -mountpoint "${mountPoint}" -nobrowse -noverify -quiet`
                    );

                    // Find and copy the .app
                    const result = await this.#shell.run(`ls "${mountPoint}"`);
                    const appName = result.stdOut.split('\n').find((f) => f.endsWith('.app'));

                    if (appName) {
                        const destAppPath = await Neutralino.filesystem.getJoinedPath(destDir, appName);
                        const srcAppPath = `${mountPoint}/${appName}`;

                        // Copy the .app bundle (use -p to preserve permissions)
                        await this.#shell.run(`cp -Rp "${srcAppPath}" "${destDir}/"`);

                        // Remove quarantine attribute to prevent "Chromium is damaged" error
                        await this.#shell.run(`xattr -rd com.apple.quarantine "${destAppPath}"`);

                        // Ensure the main executable has execute permission
                        const chromiumExecPath = await Neutralino.filesystem.getJoinedPath(
                            destAppPath,
                            'Contents',
                            'MacOS',
                            'Chromium'
                        );
                        try {
                            await this.#shell.run(`chmod +x "${chromiumExecPath}"`);
                        } catch {
                            // Try alternative executable names
                            const altNames = ['chrome', 'Google Chrome', 'BotBrowser'];
                            for (const altName of altNames) {
                                const altPath = await Neutralino.filesystem.getJoinedPath(
                                    destAppPath,
                                    'Contents',
                                    'MacOS',
                                    altName
                                );
                                try {
                                    await this.#shell.run(`chmod +x "${altPath}"`);
                                    break;
                                } catch {
                                    continue;
                                }
                            }
                        }

                        // Also set execute permission on any helper apps inside
                        const helpersPath = await Neutralino.filesystem.getJoinedPath(
                            destAppPath,
                            'Contents',
                            'Frameworks'
                        );
                        try {
                            await this.#shell.run(
                                `find "${helpersPath}" -name "*.app" -exec chmod -R +x {} \\; 2>/dev/null`
                            );
                        } catch {
                            // Helpers might not exist, ignore
                        }
                    }

                    // Unmount
                    await this.#shell.run(`hdiutil detach "${mountPoint}" -quiet`);
                } catch (error) {
                    // Try to unmount in case of error
                    try {
                        await this.#shell.run(`hdiutil detach "${mountPoint}" -force -quiet`);
                    } catch {
                        // Ignore
                    }
                    throw error;
                }
            } else if (archivePath.endsWith('.zip')) {
                // macOS: handle zip files (some releases might be zip instead of dmg)
                await this.#shell.run(`unzip -o -q "${archivePath}" -d "${destDir}"`);

                // Find .app and fix permissions
                const result = await this.#shell.run(`find "${destDir}" -name "*.app" -type d | head -1`);
                const appPath = result.stdOut.trim();
                if (appPath) {
                    // Remove quarantine attribute
                    await this.#shell.run(`xattr -rd com.apple.quarantine "${appPath}"`);
                    // Set execute permissions on all executables inside
                    await this.#shell.run(`chmod -R +x "${appPath}/Contents/MacOS"`);
                }
            }
        } else {
            // Linux: handle .deb
            console.log('Linux detected, archive ends with .deb?', archivePath.endsWith('.deb'));
            if (archivePath.endsWith('.deb')) {
                console.log('Calling extractDeb...');
                await this.#extractDeb(archivePath, destDir);
                console.log('extractDeb completed');
            } else {
                console.log('ERROR: Archive does not end with .deb!');
            }
        }
        console.log('=== EXTRACT END ===');
    }

    async #extractDeb(archivePath: string, destDir: string): Promise<void> {
        console.log('=== EXTRACT DEB START ===');
        console.log('Archive:', archivePath);
        console.log('Dest:', destDir);

        const tempDir = `${destDir}_extract_temp`;
        console.log('Temp dir:', tempDir);

        try {
            // Clean up any previous temp dir and create fresh directories
            console.log('Step 1: Creating directories...');
            await this.#shell.run(`rm -rf "${tempDir}"`);
            await this.#shell.run(`mkdir -p "${tempDir}"`);
            await this.#shell.run(`mkdir -p "${destDir}"`);
            console.log('Step 1 done: Directories created');

            // Method 1: Try dpkg-deb first (most reliable on Debian/Ubuntu)
            console.log('Step 2: Trying dpkg-deb...');
            try {
                const dpkgCmd = `dpkg-deb -x "${archivePath}" "${destDir}" 2>&1`;
                console.log('Running:', dpkgCmd);
                const dpkgResult = await this.#shell.run(dpkgCmd);
                console.log('dpkg-deb stdout:', dpkgResult.stdOut);
                console.log('dpkg-deb stderr:', dpkgResult.stdErr);

                // Check if extraction produced files
                const checkResult = await this.#shell.run(`ls "${destDir}" 2>&1`);
                console.log('ls destDir result:', checkResult.stdOut);
                if (checkResult.stdOut.includes('opt') || checkResult.stdOut.includes('usr')) {
                    console.log('dpkg-deb extraction successful!');
                    await this.#shell.run(`rm -rf "${tempDir}"`);
                    console.log('=== EXTRACT DEB SUCCESS ===');
                    return;
                }
                console.log('dpkg-deb did not produce opt/usr directories');
            } catch (e) {
                console.log('dpkg-deb failed with exception:', e);
            }

            // Method 2: Manual extraction with ar + tar
            console.log('Trying manual ar + tar extraction...');

            // Extract ar archive to temp dir
            const arResult = await this.#shell.run(`cd "${tempDir}" && ar x "${archivePath}" 2>&1`);
            console.log('ar extraction result:', arResult.stdOut, arResult.stdErr);

            // List temp dir contents
            const listResult = await this.#shell.run(`ls -la "${tempDir}" 2>&1`);
            console.log('Temp dir contents:', listResult.stdOut);

            // Find data.tar file
            const findDataResult = await this.#shell.run(`ls "${tempDir}"/data.tar* 2>/dev/null | head -1`);
            const dataTar = findDataResult.stdOut.trim();

            if (!dataTar) {
                throw new Error('No data.tar found in deb package');
            }

            console.log('Found data archive:', dataTar);

            // Extract based on file extension
            let tarCmd: string;
            if (dataTar.endsWith('.xz')) {
                tarCmd = `tar -xJf "${dataTar}" -C "${destDir}"`;
            } else if (dataTar.endsWith('.gz')) {
                tarCmd = `tar -xzf "${dataTar}" -C "${destDir}"`;
            } else if (dataTar.endsWith('.zst')) {
                tarCmd = `tar --zstd -xf "${dataTar}" -C "${destDir}"`;
            } else if (dataTar.endsWith('.bz2')) {
                tarCmd = `tar -xjf "${dataTar}" -C "${destDir}"`;
            } else {
                tarCmd = `tar -xf "${dataTar}" -C "${destDir}"`;
            }

            const tarResult = await this.#shell.run(`${tarCmd} 2>&1`);
            console.log('tar extraction result:', tarResult.stdOut, tarResult.stdErr);

            // Clean up temp dir
            await this.#shell.run(`rm -rf "${tempDir}"`);

            // Verify extraction
            const verifyResult = await this.#shell.run(`ls -la "${destDir}" 2>&1`);
            console.log('Extraction result:', verifyResult.stdOut);

            if (!verifyResult.stdOut.includes('opt') && !verifyResult.stdOut.includes('usr')) {
                throw new Error(`Extraction did not produce expected directories. Contents: ${verifyResult.stdOut}`);
            }

            console.log('Manual extraction successful');
        } catch (e) {
            // Clean up on error
            await this.#shell.run(`rm -rf "${tempDir}"`).catch(() => {});
            const errorMsg = e instanceof Error ? e.message : String(e);
            console.error('Deb extraction failed:', errorMsg);
            throw new Error(`Failed to extract deb package: ${errorMsg}`);
        }
    }

    async #extract7z(archivePath: string, destDir: string): Promise<void> {
        // Build list of possible 7z paths
        const commands: string[] = [];

        // First, try to find 7-Zip from Windows registry
        try {
            const regResult = await this.#shell.run(
                'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\7-Zip" /v Path 2>nul'
            );
            const match = regResult.stdOut.match(/Path\s+REG_SZ\s+(.+)/);
            if (match && match[1]) {
                const regPath = match[1].trim();
                commands.push(`"${regPath}\\7z.exe" x "${archivePath}" -o"${destDir}" -y`);
                console.log('Found 7-Zip in registry:', regPath);
            }
        } catch {
            // Registry query failed, continue with other paths
        }

        // Also try 32-bit registry on 64-bit Windows
        try {
            const regResult = await this.#shell.run(
                'reg query "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\7-Zip" /v Path 2>nul'
            );
            const match = regResult.stdOut.match(/Path\s+REG_SZ\s+(.+)/);
            if (match && match[1]) {
                const regPath = match[1].trim();
                commands.push(`"${regPath}\\7z.exe" x "${archivePath}" -o"${destDir}" -y`);
                console.log('Found 7-Zip in WOW6432Node registry:', regPath);
            }
        } catch {
            // Registry query failed, continue with other paths
        }

        // Add fallback paths
        commands.push(
            `7z x "${archivePath}" -o"${destDir}" -y`,
            `"C:\\Program Files\\7-Zip\\7z.exe" x "${archivePath}" -o"${destDir}" -y`,
            `"C:\\Program Files (x86)\\7-Zip\\7z.exe" x "${archivePath}" -o"${destDir}" -y`
        );

        let lastError = '';

        for (const cmd of commands) {
            try {
                console.log('Trying 7z command:', cmd);
                // Use spawnProcess instead of execCommand to avoid blocking
                const exitCode = await this.#shell.exec(cmd);
                console.log('7z extraction completed with exit code:', exitCode);

                if (exitCode === 0) {
                    // Verify extraction succeeded by checking if directory has contents
                    const checkResult = await this.#shell.run(`dir /b "${destDir}" 2>nul`);
                    if (checkResult.stdOut.trim().length > 0) {
                        return; // Success
                    }
                    lastError = 'Extraction completed but no files found';
                } else {
                    lastError = `7z exited with code ${exitCode}`;
                }
            } catch (e) {
                lastError = e instanceof Error ? e.message : String(e);
                continue;
            }
        }

        throw new Error(`7-Zip extraction failed: ${lastError}. Please install 7-Zip from https://7-zip.org/`);
    }

    async #findExecutable(installDir: string): Promise<string> {
        const osInfo = await Neutralino.computer.getOSInfo();

        if (osInfo.name.includes('Windows')) {
            // Look for chrome.exe or chromium.exe in root directory
            const possiblePaths = ['chrome.exe', 'chromium.exe', 'BotBrowser.exe'];
            for (const exe of possiblePaths) {
                const exePath = await Neutralino.filesystem.getJoinedPath(installDir, exe);
                try {
                    await Neutralino.filesystem.getStats(exePath);
                    return exePath;
                } catch {
                    continue;
                }
            }

            // Search recursively for any of the possible executables
            for (const exe of possiblePaths) {
                try {
                    const result = await this.#shell.run(`dir /s /b "${installDir}\\${exe}" 2>nul`);
                    const lines = result.stdOut.trim().split('\n').map((l) => l.replace(/\r/g, '').trim()).filter(Boolean);
                    for (const candidate of lines) {
                        try {
                            await Neutralino.filesystem.getStats(candidate);
                            return candidate;
                        } catch {
                            continue;
                        }
                    }
                } catch {
                    continue;
                }
            }

            // List directory contents for debugging
            let dirContents = '';
            try {
                const listResult = await this.#shell.run(`dir /s /b "${installDir}" 2>nul`);
                dirContents = listResult.stdOut.substring(0, 500);
            } catch {
                dirContents = 'Could not list directory';
            }

            throw new Error(`Could not find executable in ${installDir}. Contents: ${dirContents}`);
        } else if (osInfo.name.includes('Darwin')) {
            // Look for .app bundle
            const result = await this.#shell.run(
                `find "${installDir}" -maxdepth 2 -name "*.app" -type d | head -1`
            );
            const appPath = result.stdOut.trim();
            if (appPath) {
                // Verify the executable inside .app exists
                const possibleExecNames = ['Chromium', 'chrome', 'Google Chrome', 'BotBrowser'];
                for (const execName of possibleExecNames) {
                    const execPath = await Neutralino.filesystem.getJoinedPath(appPath, 'Contents', 'MacOS', execName);
                    try {
                        await Neutralino.filesystem.getStats(execPath);
                        // Ensure it has execute permission
                        await this.#shell.run(`chmod +x "${execPath}"`);
                        return appPath; // Return .app path, launcher will handle Contents/MacOS/...
                    } catch {
                        continue;
                    }
                }

                // If no known executable found, check what's in Contents/MacOS
                const macosResult = await this.#shell.run(
                    `ls "${appPath}/Contents/MacOS" 2>/dev/null | head -1`
                );
                const mainExec = macosResult.stdOut.trim();
                if (mainExec) {
                    const execPath = await Neutralino.filesystem.getJoinedPath(appPath, 'Contents', 'MacOS', mainExec);
                    await this.#shell.run(`chmod +x "${execPath}"`);
                    return appPath;
                }

                throw new Error(`Found .app bundle at ${appPath} but could not locate executable inside`);
            }
            throw new Error('Could not find .app bundle in installed kernel');
        } else {
            // Linux: .deb extracts to opt/chromium.org/chromium/
            // Look for chrome or chromium-browser executable
            const searchNames = ['chrome', 'chromium-browser', 'chromium'];
            for (const name of searchNames) {
                // First find by name (without -executable since permissions may not be set)
                const result = await this.#shell.run(
                    `find "${installDir}" -type f -name "${name}" 2>/dev/null | head -1`
                );
                const exePath = result.stdOut.trim();
                if (exePath) {
                    // Set execute permission
                    await this.#shell.run(`chmod +x "${exePath}"`);
                    return exePath;
                }
            }
            // List directory contents for debugging
            const dirContents = await this.#shell.run(`find "${installDir}" -type f 2>/dev/null | head -20`);
            throw new Error(`Could not find executable in installed kernel. Found files: ${dirContents.stdOut.trim()}`);
        }
    }

    async #loadInstalledKernels(): Promise<void> {
        try {
            const dataPath = await Neutralino.os.getPath('data');
            const filePath = await Neutralino.filesystem.getJoinedPath(dataPath, AppName, KERNELS_FILE);
            const content = await Neutralino.filesystem.readFile(filePath);
            this.#installedKernels = JSON.parse(content);

            // Validate installed kernels still exist
            const validKernels: InstalledKernel[] = [];
            for (const kernel of this.#installedKernels) {
                try {
                    await Neutralino.filesystem.getStats(kernel.executablePath);
                    validKernels.push(kernel);
                } catch {
                    // Kernel no longer exists, skip it
                }
            }
            this.#installedKernels = validKernels;
        } catch {
            this.#installedKernels = [];
        }
    }

    async #saveInstalledKernels(): Promise<void> {
        const dataPath = await Neutralino.os.getPath('data');
        const appDir = await Neutralino.filesystem.getJoinedPath(dataPath, AppName);

        try {
            await Neutralino.filesystem.getStats(appDir);
        } catch {
            await Neutralino.filesystem.createDirectory(appDir);
        }

        const filePath = await Neutralino.filesystem.getJoinedPath(appDir, KERNELS_FILE);
        await Neutralino.filesystem.writeFile(filePath, JSON.stringify(this.#installedKernels, null, 2));
        this.installedKernelsVersion.update((v) => v + 1);
    }
}
