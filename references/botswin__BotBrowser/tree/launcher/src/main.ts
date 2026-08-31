import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';
import * as Neutralino from '@neutralinojs/lib';

Neutralino.init();
Neutralino.events.on('windowClose', () => Neutralino.app.exit());

// Maximize the window on startup (guarantees maximize even when useSavedState restores a smaller size)
Neutralino.events.on('ready', () => {
    Neutralino.window.maximize().catch((err: unknown) => console.warn('maximize failed:', err));
});

// macOS: enable Cmd+C/V/X/A keyboard shortcuts
Neutralino.events.on('ready', () => {
    if (NL_OS !== 'Darwin') return;

    // Show native Edit menu
    Neutralino.window
        .setMainMenu([
            {
                text: 'Edit',
                menuItems: [
                    { id: 'undo', text: 'Undo', action: 'undo:', shortcut: 'z' },
                    { id: 'redo', text: 'Redo', action: 'redo:', shortcut: 'Z' },
                    { id: 'sep1', text: '-' },
                    { id: 'cut', text: 'Cut', action: 'cut:', shortcut: 'x' },
                    { id: 'copy', text: 'Copy', action: 'copy:', shortcut: 'c' },
                    { id: 'paste', text: 'Paste', action: 'paste:', shortcut: 'v' },
                    { id: 'selectAll', text: 'Select All', action: 'selectAll:', shortcut: 'a' },
                ],
            },
        ])
        .catch((err: unknown) => console.warn('setMainMenu failed:', err));

    // JS fallback for Cmd+C/X/V (in case native menu shortcuts don't reach the webview)
    document.addEventListener('keydown', async (e) => {
        if (!e.metaKey) return;

        if (e.key === 'c' || e.key === 'x') {
            // Copy / Cut
            const selection = window.getSelection()?.toString();
            if (!selection) return;
            e.preventDefault();
            try {
                await Neutralino.clipboard.writeText(selection);
                if (e.key === 'x') {
                    // Cut: remove selected text
                    const el = document.activeElement as HTMLInputElement | HTMLTextAreaElement;
                    if (el && 'value' in el && el.selectionStart != null && el.selectionEnd != null) {
                        const start = el.selectionStart;
                        const end = el.selectionEnd;
                        el.value = el.value.slice(0, start) + el.value.slice(end);
                        el.selectionStart = el.selectionEnd = start;
                        el.dispatchEvent(new Event('input', { bubbles: true }));
                    }
                }
            } catch {
                /* clipboard unavailable */
            }
        } else if (e.key === 'v') {
            // Paste
            const el = document.activeElement as HTMLInputElement | HTMLTextAreaElement;
            if (!el || !('value' in el)) return;
            e.preventDefault();
            try {
                const text = await Neutralino.clipboard.readText();
                const start = el.selectionStart ?? el.value.length;
                const end = el.selectionEnd ?? el.value.length;
                el.value = el.value.slice(0, start) + text + el.value.slice(end);
                el.selectionStart = el.selectionEnd = start + text.length;
                el.dispatchEvent(new Event('input', { bubbles: true }));
            } catch {
                /* clipboard unavailable */
            }
        }
    });
});

bootstrapApplication(AppComponent, appConfig).catch((err) => console.error(err));
