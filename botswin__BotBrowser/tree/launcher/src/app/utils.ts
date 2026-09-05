import * as Neutralino from '@neutralinojs/lib';
import { AppName } from './const';
import type { ShellService } from './shared/shell.service';

export function formatProxyDisplay(proxyUrl?: string): string {
    if (!proxyUrl) return '';
    try {
        // Add scheme if missing for URL parsing
        const urlString = proxyUrl.includes('://') ? proxyUrl : `http://${proxyUrl}`;
        const url = new URL(urlString);
        // Only show host:port for brevity
        return `${url.hostname}:${url.port}`;
    } catch {
        // Fallback: try to extract host:port from string
        const match = proxyUrl.match(/@?(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}):(\d+)/);
        if (match) {
            return `${match[1]}:${match[2]}`;
        }
        return proxyUrl.length > 20 ? proxyUrl.slice(0, 20) + '...' : proxyUrl;
    }
}

export function formatDateTime(ts?: number | Date | null): string {
    if (!ts) return '';
    const date = typeof ts === 'number' ? new Date(ts) : ts;
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');

    return `${month}-${day} ${hours}:${minutes}`;
}

export async function createDirectoryIfNotExists(path: string): Promise<void> {
    try {
        await Neutralino.filesystem.getStats(path);
    } catch {
        await Neutralino.filesystem.createDirectory(path);
    }
}

export async function getAppDataPath(subPath?: string): Promise<string> {
    const systemDataPath = await Neutralino.os.getPath('data');
    const pathSegments = [systemDataPath, AppName, subPath].filter((segment): segment is string => Boolean(segment));
    const fullPath = await Neutralino.filesystem.getJoinedPath(...pathSegments);
    await createDirectoryIfNotExists(fullPath);
    return fullPath;
}

export async function compressFolder(folderPath: string, outputPath: string, shell: ShellService): Promise<void> {
    try {
        const osInfo = await Neutralino.computer.getOSInfo();
        const osType = osInfo.name;

        let command;
        if (osType.includes('Linux') || osType.includes('Darwin')) {
            // Extract the parent directory and folder name
            const folderName = folderPath.split('/').pop();
            const parentPath = folderPath.substring(0, folderPath.lastIndexOf('/'));

            // Change to the parent directory and zip the folder by its name
            command = `(cd "${parentPath}" && zip -r "${outputPath}" "${folderName}")`;
        } else if (osType.includes('Windows')) {
            // Use PowerShell on Windows
            const parentPath = folderPath.substring(0, folderPath.lastIndexOf('\\'));
            const folderName = folderPath.split('\\').pop();
            command = `powershell -Command "Set-Location -Path '${parentPath}'; Compress-Archive -Path '${folderName}\\*' -DestinationPath '${outputPath}'"`;
        } else {
            throw new Error('Unsupported operating system');
        }

        const response = await shell.run(command);
        console.log('Command output: ', response.stdOut);
    } catch (error) {
        console.error('Error during folder compression: ', error);
    }
}

export async function decompressZip(zipPath: string, outputFolder: string, shell: ShellService): Promise<void> {
    try {
        // Get the OS information
        const osInfo = await Neutralino.computer.getOSInfo();
        const osType = osInfo.name;

        let command;
        if (osType.includes('Windows')) {
            // On Windows, use PowerShell's Expand-Archive command
            command = `powershell -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${outputFolder}' -Force"`;
        } else if (osType.includes('Linux') || osType.includes('Darwin')) {
            // On macOS or Linux, use unzip command
            command = `unzip -o '${zipPath}' -d '${outputFolder}'`;
        } else {
            throw new Error('Unsupported operating system');
        }

        const response = await shell.run(command);
        console.log('Command output: ', response.stdOut);
    } catch (error) {
        console.error('Error during decompression: ', error);
    }
}

export function isValidUrl(url: string): boolean {
    try {
        new URL(url);
        return true;
    } catch {
        return false;
    }
}

export function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
