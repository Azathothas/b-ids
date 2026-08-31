import { inject, Injectable, signal } from '@angular/core';
import * as Neutralino from '@neutralinojs/lib';
import { ShellService } from './shell.service';
import { AppName } from '../const';

const GITHUB_API_URL = 'https://api.github.com/repos/botswin/BotBrowser/commits?path=launcher&sha=main&per_page=1';
const REPO_ZIP_URL = 'https://github.com/botswin/BotBrowser/archive/refs/heads/main.zip';
const COMMIT_FILE = 'launcher-commit';

export type UpdateStatus = 'idle' | 'checking' | 'downloading' | 'building' | 'ready' | 'error';

@Injectable({ providedIn: 'root' })
export class UpdateService {
    readonly #shell = inject(ShellService);
    readonly status = signal<UpdateStatus>('idle');
    readonly errorMessage = signal('');
    readonly currentVersion = signal('');

    #commitFilePath = '';
    #installDir = '';
    #nodeDir = '';
    #repoDir = '';
    #stagingRepoDir = '';
    #liveDistDir = '';
    #stagingDistDir = '';
    #exeName = '';
    #isWindows = false;
    #intervalId: ReturnType<typeof setInterval> | null = null;
    // Set once a staged build is ready to be swapped in on restart.
    #pendingCommit: string | null = null;

    /** Start checking for updates: once immediately, then on an interval (default: hourly). */
    startPeriodicCheck(intervalMs = 60 * 60 * 1000): void {
        if (this.#intervalId) return;
        this.checkForUpdate().catch(console.error);
        this.#intervalId = setInterval(() => {
            // Only check if idle (skip if already updating or update ready)
            if (this.status() === 'idle') {
                this.checkForUpdate().catch(console.error);
            }
        }, intervalMs);
    }

    async checkForUpdate(): Promise<void> {
        if (this.status() !== 'idle') return;

        try {
            this.status.set('checking');

            await this.#detectPaths();
            const localCommit = await this.#readLocalCommit();
            if (localCommit) {
                this.currentVersion.set(localCommit.slice(0, 7));
            }
            const remoteCommit = await this.#fetchRemoteCommit();

            if (!remoteCommit) {
                this.status.set('idle');
                return;
            }

            if (!localCommit) {
                // First run — no commit file yet. Save current remote commit and skip update.
                console.log('First run: saving current commit hash.');
                await this.#writeLocalCommit(remoteCommit);
                this.currentVersion.set(remoteCommit.slice(0, 7));
                this.status.set('idle');
                return;
            }

            if (localCommit === remoteCommit) {
                console.log('Launcher is up to date.');
                this.status.set('idle');
                return;
            }

            console.log(`Update available: ${localCommit.slice(0, 7)} → ${remoteCommit.slice(0, 7)}`);
            await this.#performUpdate(remoteCommit);
        } catch (error) {
            console.error('Update check failed:', error);
            this.errorMessage.set(error instanceof Error ? error.message : String(error));
            this.status.set('error');

            // Reset to idle after 10 seconds so it doesn't block the UI
            setTimeout(() => {
                this.status.set('idle');
                this.errorMessage.set('');
            }, 10000);
        }
    }

    async #detectPaths(): Promise<void> {
        const osInfo = await Neutralino.computer.getOSInfo();
        this.#isWindows = osInfo.name.includes('Windows');

        if (this.#isWindows) {
            const result = await this.#shell.run('echo %LOCALAPPDATA%');
            const localAppData = result.stdOut.trim();
            this.#installDir = `${localAppData}\\${AppName}`;
            this.#nodeDir = `${this.#installDir}\\node`;
            this.#repoDir = `${this.#installDir}\\${AppName}`;
            this.#stagingRepoDir = `${this.#installDir}\\${AppName}-update`;
            this.#exeName = 'BotBrowserLauncher-win_x64.exe';
        } else {
            const result = await this.#shell.run('echo $HOME');
            const home = result.stdOut.trim();
            this.#installDir = `${home}/.botbrowser`;
            this.#nodeDir = `${this.#installDir}/node`;
            this.#repoDir = `${this.#installDir}/${AppName}`;
            this.#stagingRepoDir = `${this.#installDir}/${AppName}-update`;
            this.#exeName = await this.#detectUnixExeName(osInfo.name);
        }

        const sepR = this.#isWindows ? '\\' : '/';
        const distSub = `launcher${sepR}dist${sepR}BotBrowserLauncher`;
        this.#liveDistDir = `${this.#repoDir}${sepR}${distSub}`;
        this.#stagingDistDir = `${this.#stagingRepoDir}${sepR}${distSub}`;

        const systemDataPath = await Neutralino.os.getPath('data');
        const appDataPath = `${systemDataPath}${sepR}${AppName}`;
        try {
            await Neutralino.filesystem.getStats(appDataPath);
        } catch {
            await Neutralino.filesystem.createDirectory(appDataPath);
        }
        this.#commitFilePath = `${appDataPath}${sepR}${COMMIT_FILE}`;
    }

    async #detectUnixExeName(osName: string): Promise<string> {
        const isMac = osName.includes('Darwin') || osName.includes('macOS');
        let arch = '';
        try {
            arch = (await this.#shell.run('uname -m')).stdOut.trim();
        } catch {
            arch = 'x86_64';
        }
        if (isMac) return arch === 'arm64' ? 'BotBrowserLauncher-mac_arm64' : 'BotBrowserLauncher-mac_x64';
        if (arch === 'aarch64' || arch === 'arm64') return 'BotBrowserLauncher-linux_arm64';
        if (arch === 'armv7l' || arch === 'armhf') return 'BotBrowserLauncher-linux_armhf';
        return 'BotBrowserLauncher-linux_x64';
    }

    async #readLocalCommit(): Promise<string | null> {
        try {
            const content = await Neutralino.filesystem.readFile(this.#commitFilePath);
            return content.trim() || null;
        } catch {
            return null;
        }
    }

    async #fetchRemoteCommit(): Promise<string | null> {
        try {
            const response = await fetch(GITHUB_API_URL, {
                headers: { Accept: 'application/vnd.github.v3+json' },
            });
            if (!response.ok) return null;
            const data = await response.json();
            // Response is an array of commits (filtered by path=launcher)
            if (Array.isArray(data) && data.length > 0) {
                return data[0].sha ?? null;
            }
            return null;
        } catch {
            return null;
        }
    }

    async #writeLocalCommit(commit: string): Promise<void> {
        await Neutralino.filesystem.writeFile(this.#commitFilePath, commit);
    }

    async #performUpdate(remoteCommit: string): Promise<void> {
        // Build the new version into a STAGING dir so the running app's exe/resources
        // are never overwritten in place (that leaves a corrupt install on Windows).
        this.status.set('downloading');
        const zipPath = this.#isWindows ? `${this.#installDir}\\botbrowser-update.zip` : `${this.#installDir}/botbrowser-update.zip`;

        if (this.#isWindows) {
            await this.#shell.exec(
                `powershell -Command "$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '${REPO_ZIP_URL}' -OutFile '${zipPath}' -UseBasicParsing"`
            );
        } else {
            await this.#shell.exec(`curl -fsSL "${REPO_ZIP_URL}" -o "${zipPath}"`);
        }

        // Clear any previous staging, extract fresh into staging (NOT the live repo).
        if (this.#isWindows) {
            await this.#shell.exec(`rmdir /s /q "${this.#stagingRepoDir}"`).catch(() => {});
            await this.#shell.exec(
                `powershell -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${this.#installDir}' -Force"`
            );
            await this.#shell.exec(`rmdir /s /q "${this.#installDir}\\BotBrowser-main-old"`).catch(() => {});
            await this.#shell.exec(`rename "${this.#installDir}\\BotBrowser-main" "${AppName}-update"`);
            await this.#shell.exec(`del /f "${zipPath}"`);
        } else {
            await this.#shell.exec(`rm -rf "${this.#stagingRepoDir}"`).catch(() => {});
            await this.#shell.exec(`unzip -q -o "${zipPath}" -d "${this.#installDir}"`);
            await this.#shell.exec(`mv "${this.#installDir}/BotBrowser-main" "${this.#stagingRepoDir}"`);
            await this.#shell.exec(`rm -f "${zipPath}"`);
        }

        // npm ci + npm run build inside staging — nothing live is touched, so neu build
        // can regenerate the exe + resources.neu without hitting a locked running binary.
        this.status.set('building');
        const launcherDir = this.#isWindows ? `${this.#stagingRepoDir}\\launcher` : `${this.#stagingRepoDir}/launcher`;
        const npmCmd = this.#isWindows ? `"${this.#nodeDir}\\npm.cmd"` : `"${this.#nodeDir}/bin/npm"`;
        const npmCacheDir = this.#isWindows ? `${this.#installDir}\\npm-cache` : `${this.#installDir}/npm-cache`;
        const pathEnv = this.#isWindows
            ? `set "PATH=${this.#nodeDir};%PATH%" && set "NPM_CONFIG_CACHE=${npmCacheDir}" && set "NPM_CONFIG_UPDATE_NOTIFIER=false" &&`
            : `export PATH="${this.#nodeDir}/bin:$PATH" NPM_CONFIG_CACHE="${npmCacheDir}" NPM_CONFIG_UPDATE_NOTIFIER=false &&`;

        if (this.#isWindows) {
            await this.#shell.exec(`if not exist "${npmCacheDir}" mkdir "${npmCacheDir}"`);
        } else {
            await this.#shell.exec(`mkdir -p "${npmCacheDir}"`);
        }
        await this.#shell.exec(`${pathEnv} cd "${launcherDir}" && ${npmCmd} ci && ${npmCmd} run build`);

        // Verify the staged build actually produced a runnable app before offering to apply it.
        const sep = this.#isWindows ? '\\' : '/';
        const stagedExe = `${this.#stagingDistDir}${sep}${this.#exeName}`;
        const stagedResources = `${this.#stagingDistDir}${sep}resources.neu`;
        const ok = (await this.#exists(stagedExe)) && (await this.#exists(stagedResources));
        if (!ok) {
            await this.#shell.exec(this.#isWindows ? `rmdir /s /q "${this.#stagingRepoDir}"` : `rm -rf "${this.#stagingRepoDir}"`).catch(() => {});
            throw new Error('Update build did not produce a valid app bundle; the current install was left untouched.');
        }

        // Staged and verified. Do NOT touch the live install or the commit file yet — the swap
        // happens on restart (applyUpdateAndRestart), after this process exits and releases locks.
        this.#pendingCommit = remoteCommit;
        this.status.set('ready');
    }

    async #exists(path: string): Promise<boolean> {
        try {
            await Neutralino.filesystem.getStats(path);
            return true;
        } catch {
            return false;
        }
    }

    /**
     * Apply a staged update: spawn a detached script that waits for this process to exit,
     * swaps the staged app bundle over the live one (now unlocked), records the new commit,
     * and relaunches. Falls back to a plain restart if no update is staged.
     */
    async applyUpdateAndRestart(): Promise<void> {
        if (!this.#pendingCommit) {
            Neutralino.app.restartProcess();
            return;
        }
        try {
            const pid = (globalThis as any).NL_PID as number | undefined;
            const scriptPath = await this.#writeSwapScript(this.#pendingCommit, pid);
            if (this.#isWindows) {
                await Neutralino.os.spawnProcess(`cmd /c start "" /min "${scriptPath}"`);
            } else {
                await Neutralino.os.spawnProcess(`/bin/sh -c "nohup bash '${scriptPath}' >/dev/null 2>&1 &"`);
            }
            // Give the detached process a moment to start watching, then exit to release file locks.
            setTimeout(() => Neutralino.app.exit(), 400);
        } catch (error) {
            console.error('Failed to apply update, falling back to restart:', error);
            Neutralino.app.restartProcess();
        }
    }

    async #writeSwapScript(sha: string, pid: number | undefined): Promise<string> {
        const sep = this.#isWindows ? '\\' : '/';
        const liveExe = `${this.#liveDistDir}${sep}${this.#exeName}`;
        if (this.#isWindows) {
            const path = `${this.#installDir}\\update-apply.bat`;
            const waitBlock = pid
                ? `set /a n=0\n:wait\ntasklist /fi "PID eq ${pid}" 2>nul | find "${pid}" >nul\nif not errorlevel 1 (\n  set /a n+=1\n  if %n% lss 120 ( ping -n 2 127.0.0.1 >nul & goto wait )\n)`
                : `ping -n 3 127.0.0.1 >nul`;
            const script =
                `@echo off\r\n` +
                `${waitBlock}\r\n` +
                `if not exist "${this.#stagingDistDir}\\${this.#exeName}" goto relaunch\r\n` +
                `rmdir /s /q "${this.#liveDistDir}.old" 2>nul\r\n` +
                `move "${this.#liveDistDir}" "${this.#liveDistDir}.old" >nul 2>&1\r\n` +
                `move "${this.#stagingDistDir}" "${this.#liveDistDir}" >nul 2>&1\r\n` +
                `if exist "${liveExe}" (\r\n` +
                `  rmdir /s /q "${this.#liveDistDir}.old" 2>nul\r\n` +
                `  > "${this.#commitFilePath}" echo ${sha}\r\n` +
                `) else (\r\n` +
                `  if exist "${this.#liveDistDir}.old" move "${this.#liveDistDir}.old" "${this.#liveDistDir}" >nul 2>&1\r\n` +
                `)\r\n` +
                `rmdir /s /q "${this.#stagingRepoDir}" 2>nul\r\n` +
                `:relaunch\r\n` +
                `start "" "${liveExe}"\r\n` +
                `del /f /q "%~f0"\r\n`;
            await Neutralino.filesystem.writeFile(path, script);
            return path;
        }
        const path = `${this.#installDir}/update-apply.sh`;
        const waitBlock = pid
            ? `i=0\nwhile kill -0 ${pid} 2>/dev/null; do sleep 1; i=$((i+1)); [ $i -gt 120 ] && break; done`
            : `sleep 2`;
        const script =
            `#!/bin/bash\n` +
            `${waitBlock}\n` +
            `if [ -e "${this.#stagingDistDir}/${this.#exeName}" ]; then\n` +
            `  rm -rf "${this.#liveDistDir}.old"\n` +
            `  mv "${this.#liveDistDir}" "${this.#liveDistDir}.old" 2>/dev/null\n` +
            `  mv "${this.#stagingDistDir}" "${this.#liveDistDir}" 2>/dev/null\n` +
            `  if [ -e "${liveExe}" ]; then\n` +
            `    rm -rf "${this.#liveDistDir}.old"\n` +
            `    echo "${sha}" > "${this.#commitFilePath}"\n` +
            `  else\n` +
            `    [ -d "${this.#liveDistDir}.old" ] && mv "${this.#liveDistDir}.old" "${this.#liveDistDir}"\n` +
            `  fi\n` +
            `  rm -rf "${this.#stagingRepoDir}"\n` +
            `fi\n` +
            `chmod +x "${liveExe}" 2>/dev/null\n` +
            `nohup "${liveExe}" >/dev/null 2>&1 &\n` +
            `rm -- "$0"\n`;
        await Neutralino.filesystem.writeFile(path, script);
        return path;
    }
}
