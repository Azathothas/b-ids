export type BotProfileBasicInfo = {
    version: string;
    userAgent: string;
    unmaskedVendor: string;
    unmaskedRenderer: string;
};

export function tryParseBotProfile(data: string): BotProfileBasicInfo | null {
    try {
        const info = JSON.parse(data);
        if (info && typeof info.userAgent === 'string' && info.userAgent) {
            return {
                version: info.version ?? '',
                userAgent: info.userAgent,
                unmaskedVendor: info.unmaskedVendor ?? '',
                unmaskedRenderer: info.unmaskedRenderer ?? '',
            };
        }
        return null;
    } catch {
        return null;
    }
}

export function extractMajorVersion(versionOrUserAgent: string): number | null {
    // Try to match Chrome/Chromium version pattern: Chrome/144.0.7559.97
    const chromeMatch = versionOrUserAgent.match(/Chrom(?:e|ium)\/(\d+)/);
    if (chromeMatch && chromeMatch[1]) {
        return parseInt(chromeMatch[1], 10);
    }

    // Try to match version string like "144.0.7559.97"
    const versionMatch = versionOrUserAgent.match(/^(\d+)\./);
    if (versionMatch && versionMatch[1]) {
        return parseInt(versionMatch[1], 10);
    }

    return null;
}

// WebKit/Safari profiles still run on a Chromium kernel but their UA carries no Chrome major.
export function isWebKitProfile(basicInfo: BotProfileBasicInfo): boolean {
    const ua = basicInfo.userAgent || '';
    const hasSafariMarker = /AppleWebKit\/|Safari\//.test(ua);
    const hasChrome = /Chrom(?:e|ium)\//.test(ua);
    return hasSafariMarker && !hasChrome;
}

// Map a WebKit/Safari UA's Safari major (Version/<N>) to the Chromium kernel major required by that profile family.
// Source: profiles/stable/README.md (webkit26_* profiles require v149 BotBrowser binary).
const WEBKIT_TO_CHROMIUM_MAJOR: Record<number, number> = {
    26: 149,
};

export function getRequiredKernelMajor(userAgent: string): number | null {
    if (!userAgent || /Chrom(?:e|ium)\//.test(userAgent)) return null;
    if (!/AppleWebKit\/|Safari\//.test(userAgent)) return null;
    const m = userAgent.match(/Version\/(\d+)/);
    if (!m) return null;
    return WEBKIT_TO_CHROMIUM_MAJOR[parseInt(m[1]!, 10)] ?? null;
}
