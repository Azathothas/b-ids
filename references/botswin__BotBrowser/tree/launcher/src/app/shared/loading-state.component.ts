import { CommonModule } from '@angular/common';
import { Component, Input } from '@angular/core';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
    selector: 'app-loading-state',
    standalone: true,
    imports: [CommonModule, MatProgressBarModule, MatProgressSpinnerModule, MatIconModule],
    template: `
        @if (loading) {
            <div class="loading-container" [class.overlay]="overlay">
                @if (mode === 'bar') {
                    <mat-progress-bar mode="indeterminate"></mat-progress-bar>
                } @else {
                    <mat-spinner [diameter]="spinnerSize"></mat-spinner>
                    @if (message) {
                        <p class="loading-message">{{ message }}</p>
                    }
                }
            </div>
        } @else if (error) {
            <div class="error-container">
                <mat-icon>error</mat-icon>
                <span>{{ error }}</span>
            </div>
        } @else if (empty) {
            <div class="empty-container" [class.small]="emptySize === 'small'">
                <mat-icon>{{ emptyIcon }}</mat-icon>
                <p>{{ emptyMessage }}</p>
                @if (emptyHint) {
                    <p class="hint">{{ emptyHint }}</p>
                }
            </div>
        }
    `,
    styles: [
        `
            .loading-container {
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                padding: 48px 16px;

                &.overlay {
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(255, 255, 255, 0.8);
                    z-index: 10;
                    padding: 0;
                }

                mat-progress-bar {
                    width: 100%;
                }

                .loading-message {
                    margin-top: 16px;
                    color: rgba(0, 0, 0, 0.54);
                    font-size: 14px;
                }
            }

            .error-container {
                display: flex;
                align-items: center;
                gap: 8px;
                padding: 12px 16px;
                background: #ffebee;
                color: #c62828;

                mat-icon {
                    font-size: 20px;
                    width: 20px;
                    height: 20px;
                }
            }

            .empty-container {
                padding: 48px 16px;
                text-align: center;
                color: rgba(0, 0, 0, 0.54);

                mat-icon {
                    font-size: 48px;
                    width: 48px;
                    height: 48px;
                    margin-bottom: 16px;
                    opacity: 0.5;
                }

                p {
                    margin: 0;
                }

                .hint {
                    font-size: 13px;
                    margin-top: 8px;
                }

                &.small {
                    padding: 24px 16px;

                    mat-icon {
                        font-size: 24px;
                        width: 24px;
                        height: 24px;
                    }
                }
            }
        `,
    ],
})
export class LoadingStateComponent {
    @Input() loading = false;
    @Input() error: string | null = null;
    @Input() empty = false;
    @Input() overlay = false;
    @Input() mode: 'bar' | 'spinner' = 'spinner';
    @Input() spinnerSize = 40;
    @Input() message = '';
    @Input() emptyIcon = 'inbox';
    @Input() emptyMessage = 'No data';
    @Input() emptyHint = '';
    @Input() emptySize: 'normal' | 'small' = 'normal';
}
