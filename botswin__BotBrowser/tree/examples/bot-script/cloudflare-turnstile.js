/**
 * ⚠️ PRIVACY RESEARCH USE ONLY
 * Run exclusively inside authorized privacy research environments.
 * See: https://github.com/botswin/BotBrowser/blob/main/DISCLAIMER.md
 */

// BotBrowser Frame Automation Script
// Target: frames with src containing "https://challenges.cloudflare.com/cdn-cgi/challenge-platform"
console.log("BotBrowser Frame Automation Script loaded!");

let targetFrameFound = false;
let monitoringActive = true;

// Check if chrome.debugger API is available
if (typeof chrome !== 'undefined' && chrome.debugger) {
    console.log("chrome.debugger API is available!");

    // Start monitoring for target frames
    startFrameMonitoring();
} else {
    console.log("ERROR: chrome.debugger API not available!");
}

function startFrameMonitoring() {
    console.log("Starting frame monitoring for https://challenges.cloudflare.com/cdn-cgi/challenge-platform");

    // Get all targets and start monitoring
    chrome.debugger.getTargets(function(targets) {
        console.log("Found " + targets.length + " targets");

        // Attach to all relevant targets
        targets.forEach(function(target) {
            if (target.type === 'page') {
                console.log("Found page target:", target.id, "URL:", target.url);
                attachToTarget(target);
            } else if (target.type === 'iframe') {
                console.log("Found iframe target:", target.id, "URL:", target.url);
                attachToTarget(target);
            } else if (target.type === 'other' && target.url && target.url.includes('challenges.cloudflare.com')) {
                console.log("Found Cloudflare challenge target:", target.type, target.id, "URL:", target.url);
                attachToTarget(target);
            } else {
                console.log("Found other target:", target.type, target.id, "URL:", target.url);
            }
        });

        // Continue monitoring for new targets every 2 seconds
        if (monitoringActive) {
            setTimeout(startFrameMonitoring, 2000);
        }
    });
}

// Track which targets we're currently working on to avoid duplicate processing
let activeTargets = new Set();

function attachToTarget(target) {
    console.log("Attempting to process target:", target.id, "URL:", target.url);

    // Skip if we're currently processing this target
    if (activeTargets.has(target.id)) {
        console.log("Target currently being processed, skipping:", target.id);
        return;
    }

    // Simplified: Focus only on Cloudflare challenge frames
    const url = target.url || '';
    const isCloudflareChallenge = url.includes('challenges.cloudflare.com/cdn-cgi/challenge-platform');

    if (!isCloudflareChallenge) {
        console.log("Not a Cloudflare challenge frame, skipping:", url);
        return;
    }

    console.log("Found Cloudflare challenge frame:", url);

    // Mark as active while we're processing it
    activeTargets.add(target.id);

    // First try to detach any existing debugger, then attach
    chrome.debugger.detach({targetId: target.id}, function() {
        // Ignore detach errors, just try to attach
        console.log("Attempted to detach existing debugger from target:", target.id);

        // Now try to attach
        chrome.debugger.attach({targetId: target.id}, "1.3", function() {
            if (chrome.runtime.lastError) {
                console.log("Failed to attach to target:", target.id, "Error:", chrome.runtime.lastError.message);
                // Remove from active targets to allow retry
                activeTargets.delete(target.id);
                return;
            }

            console.log("Successfully attached to target:", target.id);

            // Enable necessary domains (keep in active targets while processing)
            enableDomainsAndStartMonitoring(target.id);
        });
    });
}

function enableDomainsAndStartMonitoring(targetId) {
    // Simplified: Only enable necessary domains for clicking
    chrome.debugger.sendCommand({targetId: targetId}, "Runtime.enable", {}, function() {
        if (chrome.runtime.lastError) {
            console.log("Failed to enable Runtime for target:", targetId, chrome.runtime.lastError.message);
            activeTargets.delete(targetId);
            return;
        }

        console.log("Runtime enabled for target:", targetId);

        // Setup mobile emulation if needed and start clicking immediately
        setupMobileEmulation(targetId);
        
        // Start clicking with minimal delay
        setTimeout(function() {
            performDirectClick(targetId);
        }, 500);
    });
}

function setupMobileEmulation(targetId) {
    // Check if this is a mobile device/viewport first
    chrome.debugger.sendCommand({targetId: targetId}, "Runtime.evaluate", {
        expression: `
            (() => {
                // Check various mobile indicators
                const isMobile = /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) ||
                                window.innerWidth <= 768 ||
                                ('ontouchstart' in window) ||
                                (navigator.maxTouchPoints > 0);
                return {
                    isMobile: isMobile,
                    userAgent: navigator.userAgent,
                    innerWidth: window.innerWidth,
                    touchSupport: 'ontouchstart' in window,
                    maxTouchPoints: navigator.maxTouchPoints
                };
            })()
        `
    }, function(result) {
        if (result && result.result && result.result.value) {
            const deviceInfo = result.result.value;
            console.log("Device detection:", deviceInfo);

            if (deviceInfo.isMobile) {
                console.log("Mobile device detected, enabling touch events");
                // Enable touch events for mobile interaction
                chrome.debugger.sendCommand({targetId: targetId}, "Emulation.setEmitTouchEventsForMouse", {
                    enabled: true,
                    configuration: 'mobile'
                }, function() {
                    console.log("Mobile touch events enabled for target:", targetId);
                });
            } else {
                console.log("Desktop device detected, using standard mouse events");
            }
        } else {
            console.log("Could not detect device type, defaulting to desktop mode");
        }
    });
}

// Simplified direct click function - no element detection needed
function performDirectClick(targetId) {
    console.log("Performing direct coordinate click on target:", targetId);
    
    // Click at fixed coordinates (30px from left, 30px from top)
    // Based on GitHub reference approach: find frame + click at offset
    const clickX = 30;
    const clickY = 30;
    
    console.log("Clicking directly at coordinates:", clickX, clickY);
    performMouseClick(targetId, clickX, clickY);
}

// Simple mouse click function using Chrome DevTools Protocol
function performMouseClick(targetId, x, y) {
    console.log("Performing mouse click at coordinates:", x, y);

    // Perform mouse press
    chrome.debugger.sendCommand({targetId: targetId}, "Input.dispatchMouseEvent", {
        type: "mousePressed",
        x: x,
        y: y,
        button: "left",
        clickCount: 1
    }, function() {
        if (chrome.runtime.lastError) {
            console.log("Mouse press failed:", chrome.runtime.lastError.message);
            return;
        }

        console.log("Mouse pressed at:", x, y);

        // Release mouse button after short delay
        setTimeout(function() {
            chrome.debugger.sendCommand({targetId: targetId}, "Input.dispatchMouseEvent", {
                type: "mouseReleased",
                x: x,
                y: y,
                button: "left",
                clickCount: 1
            }, function() {
                if (chrome.runtime.lastError) {
                    console.log("Mouse release failed:", chrome.runtime.lastError.message);
                    return;
                }

                console.log("Mouse released. Click completed successfully!");
                console.log("Turnstile automation completed for this target!");

                // Remove this specific target from active targets to allow new attempts
                activeTargets.delete(targetId);

                // Keep monitoring active for new pages/targets
                console.log("Continuing to monitor for new Turnstile challenges...");
            });
        }, 100);
    });
}

// Global status function
function getAutomationStatus() {
    return {
        monitoring: monitoringActive,
        targetFound: targetFrameFound,
        timestamp: new Date().toISOString(),
        activeTargets: Array.from(activeTargets)
    };
}

console.log("Frame automation script initialized. Monitoring started...");
