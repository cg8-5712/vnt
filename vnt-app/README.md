# VNT App

Tauri desktop shell for VNT2.

The UI is shared with `../vnt-web/frontend`. Development loads the Vite dev server, and production bundles `../vnt-web/static`.

```bash
yarn
yarn dev
yarn build
```

Android bootstrap:

```bash
yarn android:init
yarn android:dev
yarn android:build
```

Notes:

- Desktop keeps the current tray/window shell behavior.
- Mobile loads the shared UI from the embedded localhost HTTP server instead of the Vite dev server.
- Full Android VPN mode still requires `VpnService`/TUN fd wiring on the runtime path.
