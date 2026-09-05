export type KernelPlatform = 'win_x86_64' | 'mac_arm64' | 'mac_x86_64' | 'linux_x86_64' | 'linux_arm64';

export interface KernelAsset {
    id: number; // GitHub asset ID, globally unique
    name: string;
    platform: KernelPlatform;
    downloadUrl: string;
    size: number;
    assetDate: string; // Date extracted from asset name, format: YYYYMMDD
}

export interface KernelRelease {
    tagName: string;
    name: string;
    majorVersion: number;
    fullVersion: string;
    publishedAt: string;
    assets: KernelAsset[];
}

export interface InstalledKernel {
    id: string;
    tagName: string;
    majorVersion: number;
    fullVersion: string;
    platform: KernelPlatform;
    installPath: string;
    executablePath: string;
    installedAt: number;
    assetName?: string; // Original asset filename
    assetDate?: string; // Date extracted from asset name, format: YYYYMMDD
}

export interface DownloadProgress {
    tagName: string;
    platform: KernelPlatform;
    downloadedBytes: number;
    totalBytes: number;
    status: 'downloading' | 'extracting' | 'completed' | 'failed';
    error?: string;
}

export function parseVersion(tagName: string): { major: number; full: string; isValidMajor: boolean } {
    // Handle formats like "144.0.7559.97" or "v142-20251117"
    const vMatch = tagName.match(/^v?(\d+)/);
    const major = vMatch && vMatch[1] ? parseInt(vMatch[1], 10) : 0;

    // Try to extract full version from tag
    const fullMatch = tagName.match(/^(\d+\.\d+\.\d+\.\d+)/);
    const full: string = fullMatch && fullMatch[1] ? fullMatch[1] : tagName.replace(/^v/, '');

    // A valid major version should be a Chrome major version (typically 100+)
    // Filter out date-like versions like "20250728" (starts with 20 and is 8 digits)
    const isDateLikeVersion = /^20\d{6}$/.test(tagName);
    const isValidMajor = major >= 100 && major < 999 && !isDateLikeVersion;

    return { major, full, isValidMajor };
}

// Compare two versions to determine which is newer (higher build number)
// Returns: positive if a > b, negative if a < b, 0 if equal
export function compareVersions(a: string, b: string): number {
    const partsA = a.split('.').map((p) => parseInt(p, 10) || 0);
    const partsB = b.split('.').map((p) => parseInt(p, 10) || 0);

    for (let i = 0; i < Math.max(partsA.length, partsB.length); i++) {
        const numA = partsA[i] || 0;
        const numB = partsB[i] || 0;
        if (numA !== numB) {
            return numA - numB;
        }
    }
    return 0;
}

// Extract date from asset name like "BotBrowser_144.0.7559.97_20250130_win_x86_64.7z"
// Returns date string in format "YYYYMMDD" or empty string if not found
export function getAssetDate(name: string): string {
    // Look for 8-digit date pattern (YYYYMMDD) in the filename
    const dateMatch = name.match(/_(\d{8})_/);
    return dateMatch && dateMatch[1] ? dateMatch[1] : '';
}

export function getPlatformFromAssetName(name: string): KernelPlatform | null {
    if (name.endsWith('.7z')) return 'win_x86_64';
    if (name.endsWith('.dmg')) return name.includes('arm64') ? 'mac_arm64' : 'mac_x86_64';
    if (name.endsWith('.deb')) return name.includes('arm64') ? 'linux_arm64' : 'linux_x86_64';
    return null;
}

// Pick the latest asset for a given platform within a release.
// Among assets matching the platform, prefers the one with the highest assetDate (YYYYMMDD).
export function getLatestAssetForPlatform(
    assets: KernelAsset[],
    platform: KernelPlatform
): KernelAsset | undefined {
    const candidates = assets.filter((a) => a.platform === platform);
    if (candidates.length === 0) return undefined;
    return candidates.reduce((latest, a) =>
        (a.assetDate || '') > (latest.assetDate || '') ? a : latest
    );
}

export function getCurrentPlatform(osName: string, arch?: string): KernelPlatform {
    const osLower = osName.toLowerCase();

    if (osLower.includes('windows')) {
        return 'win_x86_64';
    }
    if (osLower.includes('darwin') || osLower.includes('macos') || osLower.includes('mac os')) {
        // Check architecture - default to arm64 for modern Macs
        if (arch?.includes('arm') || arch?.includes('aarch')) {
            return 'mac_arm64';
        }
        return 'mac_x86_64';
    }
    // Linux distributions: Ubuntu, Debian, Fedora, CentOS, Arch, etc.
    if (
        osLower.includes('linux') ||
        osLower.includes('ubuntu') ||
        osLower.includes('debian') ||
        osLower.includes('fedora') ||
        osLower.includes('centos') ||
        osLower.includes('arch') ||
        osLower.includes('manjaro') ||
        osLower.includes('mint') ||
        osLower.includes('opensuse') ||
        osLower.includes('rhel') ||
        osLower.includes('redhat')
    ) {
        if (arch?.includes('arm') || arch?.includes('aarch')) {
            return 'linux_arm64';
        }
        return 'linux_x86_64';
    }
    return 'win_x86_64'; // fallback
}
