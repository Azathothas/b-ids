import { Injectable } from '@angular/core';
import * as Neutralino from '@neutralinojs/lib';
import { compact } from 'lodash-es';
import type { BrowserProfile } from '../data/browser-profile';
import { createDirectoryIfNotExists, getAppDataPath } from '../utils';
import { AppName } from '../const';

export const kProfileConfigFileName = 'profile-config.json';

@Injectable({ providedIn: 'root' })
export class BrowserProfileService {
    async getBasePath(): Promise<string> {
        console.log('getBasePath: calling getAppDataPath...');
        const result = await getAppDataPath('browser-profiles');
        console.log('getBasePath: result =', result);
        return result;
    }

    async getBotProfileTempOutputPath(browserProfile: string | BrowserProfile) {
        const browserProfileId = typeof browserProfile === 'string' ? browserProfile : browserProfile.id;

        const sysTempPath = await Neutralino.os.getPath('temp');
        const botProfilesBasePath = await Neutralino.filesystem.getJoinedPath(sysTempPath, AppName, 'bot-profiles');
        await createDirectoryIfNotExists(botProfilesBasePath);
        const result = await Neutralino.filesystem.getJoinedPath(botProfilesBasePath, `${browserProfileId}.json`);

        return result;
    }

    async getBrowserProfilePath(browserProfile: string | BrowserProfile) {
        const id = typeof browserProfile === 'string' ? browserProfile : browserProfile.id;

        console.log('getBrowserProfilePath: getting base path...');
        const basePath = await this.getBasePath();
        console.log('getBrowserProfilePath: base path =', basePath);

        console.log('getBrowserProfilePath: joining path...');
        const browserProfilePath = await Neutralino.filesystem.getJoinedPath(basePath, id);
        console.log('getBrowserProfilePath: browserProfilePath =', browserProfilePath);

        console.log('getBrowserProfilePath: creating directory...');
        await createDirectoryIfNotExists(browserProfilePath);
        console.log('getBrowserProfilePath: directory created/exists');

        return browserProfilePath;
    }

    async getBrowserProfileUserDataDirPath(browserProfile: string | BrowserProfile) {
        const basePath = await this.getBrowserProfilePath(browserProfile);
        const result = await Neutralino.filesystem.getJoinedPath(basePath, 'user-data-dir');
        return result;
    }

    async getBrowserProfileDiskCacheDirPath(browserProfile: string | BrowserProfile) {
        const browserProfileId = typeof browserProfile === 'string' ? browserProfile : browserProfile.id;
        const sysTempPath = await Neutralino.os.getPath('temp');
        const result = await Neutralino.filesystem.getJoinedPath(
            sysTempPath,
            AppName,
            'disk-cache-dir',
            browserProfileId
        );
        return result;
    }

    async saveBrowserProfile(browserProfile: BrowserProfile): Promise<void> {
        console.log('saveBrowserProfile: getting base path...');
        const basePath = await this.getBrowserProfilePath(browserProfile);
        console.log('saveBrowserProfile: base path =', basePath);

        console.log('saveBrowserProfile: joining path...');
        const filename = await Neutralino.filesystem.getJoinedPath(basePath, kProfileConfigFileName);
        console.log('saveBrowserProfile: filename =', filename);

        console.log('saveBrowserProfile: writing file...');
        await Neutralino.filesystem.writeFile(filename, JSON.stringify(browserProfile));
        console.log(`Browser profile saved: ${filename}`);
    }

    async getAllBrowserProfiles(): Promise<BrowserProfile[]> {
        const browserProfilePath = await this.getBasePath();
        const entries = await Neutralino.filesystem.readDirectory(browserProfilePath);

        const result = compact(
            await Promise.all(
                entries.map(async (entry) => {
                    if (entry.type == 'FILE') return;

                    try {
                        const content = await Neutralino.filesystem.readFile(
                            await Neutralino.filesystem.getJoinedPath(entry.path, kProfileConfigFileName)
                        );
                        return JSON.parse(content);
                    } catch {}
                })
            )
        );

        result.sort((a, b) => b.createdAt - a.createdAt);
        return result;
    }

    async getBrowserProfile(browserProfile: string | BrowserProfile): Promise<BrowserProfile | undefined> {
        const basePath = await this.getBrowserProfilePath(browserProfile);
        const filename = await Neutralino.filesystem.getJoinedPath(basePath, kProfileConfigFileName);
        const content = await Neutralino.filesystem.readFile(filename);

        console.log(`Browser profile loaded: ${filename}`);

        return JSON.parse(content);
    }

    async deleteBrowserProfile(browserProfile: string | BrowserProfile): Promise<void> {
        const basePath = await this.getBrowserProfilePath(browserProfile);
        await Neutralino.filesystem.remove(basePath);
    }

    async deleteBrowserProfiles(browserProfiles: (string | BrowserProfile)[]): Promise<void> {
        await Promise.all(browserProfiles.map((browserProfile) => this.deleteBrowserProfile(browserProfile)));
    }

    async getTempUserDataDirFilePath(browserProfileId: string): Promise<string> {
        const sysTempPath = await Neutralino.os.getPath('temp');
        const basePath = await Neutralino.filesystem.getJoinedPath(sysTempPath, AppName, 'user-data-dirs');
        await createDirectoryIfNotExists(basePath);
        const userDataDirFilePath = await Neutralino.filesystem.getJoinedPath(basePath, `${browserProfileId}.zip`);

        return userDataDirFilePath;
    }
}
