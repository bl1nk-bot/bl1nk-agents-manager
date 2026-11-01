Canvas apps are single-file HTML applications with specific security and environment restrictions.

## Core Restrictions
- **Client-only**: No server-side code execution.
- **Single-file**: Must be contained within a single HTML file (can use CDNs for external libraries).
- **No Database**: No built-in persistence. Use message passing to a bot or external server (if CSP disabled).
- **No LocalStorage**: Browser `localStorage` and `sessionStorage` APIs are disabled.
- **No Service/Shared Workers**: These APIs are not enabled.
- **Restricted Web Workers**: Only workers initialized via `blob://` or `data://` URLs are allowed.

## Security & Connectivity
- **Content Security Policy (CSP)**: Strict restrictions on external resources.
- **Trusted Origins**: Resources can be loaded from `cdnjs.cloudflare.com` and `cdn.jsdelivr.net`.
- **External Links**: Cross-origin links trigger a confirmation dialog.
- **Untrusted Resources**: Loading untrusted resources triggers a "Allow untrusted external resources" prompt.

## Hardware & System Access
- **No Webcam/Microphone**: Access is disabled.
- **Clipboard**: Write-only access (cannot read from clipboard).
- **No History API**: Navigation via `window.history` is not possible.
- **No Modals**: `alert()`, `confirm()`, `print()`, and `prompt()` are disabled (iframe lacks `allow-modals`).

## Platform Behavior
- **Light/Dark Mode**:
  - Chrome/Firefox: Inherits from Poe settings.
  - Safari: Inherits from OS settings, regardless of Poe settings.
- **File Downloads**: Enabled via `<a download>`. Non-trusted sources trigger a confirmation.
